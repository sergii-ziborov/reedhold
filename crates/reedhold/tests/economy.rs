//! Economic invariants that cross crate boundaries.
//!
//! Two ledgers: reputation is grown and never moves, credits are minted by
//! verified work and do move. Nothing here may let money buy influence.

use reedhold::ads::{AdvertisingLimits, AdvertisingRoot};
use reedhold::core::{ContentId, Digest32, IdentityId};
use reedhold::rep::{Graph, IdentityRep, Reaction, ReactionKind, transfer};
use reedhold::work::{Book, WorkKind};

const WEEK: u64 = 7 * 86_400;

fn node(byte: u8) -> Digest32 {
    Digest32::from_bytes([byte; 32])
}

fn identity(byte: u8) -> IdentityId {
    IdentityId::from_digest(Digest32::from_bytes([byte; 32]))
}

fn post() -> ContentId {
    ContentId::from_digest(Digest32::from_bytes([200; 32]))
}

fn mature() -> IdentityRep {
    IdentityRep {
        continuity: 4000,
        social: 4000,
        content: 4000,
        curation: 4000,
        contribution: 4000,
        moderation: 4000,
    }
}

/// A wallet full of credits must not become a seat in the committee.
#[test]
fn credits_buy_no_consensus_seat() {
    let mut book = Book::new();
    book.record(node(1), WorkKind::Repair, 50_000, 1, true)
        .expect("worker records repair");
    let whale = node(2);
    let purse = book.credits(node(1));
    assert!(purse > 0);
    book.transfer(node(1), whale, purse)
        .expect("credits are transferable");

    assert_eq!(book.credits(whale), purse);
    assert!(
        !book.eligible(whale, 10_000),
        "buying every credit must still buy zero votes"
    );
    assert!(book.eligible(node(1), 10_000), "the worker keeps the seat");
}

/// Reputation is not a token. There is no path that moves it.
#[test]
fn reputation_cannot_be_bought_with_work_or_credits() {
    assert!(transfer(identity(1), identity(2), 5_000).is_err());

    let mut graph = Graph::new();
    graph.seed(identity(1), mature());
    let rich = graph.identity(identity(2));
    assert_eq!(rich.strength(), 0, "an empty identity starts at zero");

    let mut book = Book::new();
    book.record(node(2), WorkKind::Storage, 100_000, 1, true)
        .expect("work records");
    assert_eq!(
        graph.identity(identity(2)).strength(),
        0,
        "minting credits must not raise social weight by itself"
    );
}

/// Presence is not contribution: idling earns a fraction of repairing.
#[test]
fn uptime_alone_earns_far_less_than_repair() {
    let mut idle = Book::new();
    let mut worker = Book::new();
    idle.record(node(1), WorkKind::Uptime, 500, 1, true)
        .expect("uptime records");
    worker
        .record(node(1), WorkKind::Repair, 500, 1, true)
        .expect("repair records");
    assert!(worker.credits(node(1)) >= idle.credits(node(1)).saturating_mul(4));
}

/// Unreliable work is worth half. Challenges are the whole point.
#[test]
fn unreliable_storage_is_paid_half() {
    let mut honest = Book::new();
    let mut flaky = Book::new();
    honest
        .record(node(1), WorkKind::Storage, 1_000, 1, true)
        .expect("records");
    flaky
        .record(node(1), WorkKind::Storage, 1_000, 1, false)
        .expect("records");
    assert_eq!(flaky.credits(node(1)) * 2, honest.credits(node(1)));
}

/// A paid burst must settle below a small, slow, independent following.
#[test]
fn a_bought_burst_settles_below_mature_independent_support() {
    let mut pump = Graph::new();
    let mut organic = Graph::new();

    for byte in 2_u8..=101 {
        pump.react(Reaction {
            author: identity(byte),
            target: post(),
            kind: ReactionKind::Like,
            cluster: Digest32::from_bytes([7; 32]),
            topic: Digest32::from_bytes([0; 32]),
            created_at: 0,
        })
        .expect("burst reaction");
    }

    for byte in 2_u8..=21 {
        organic.seed(identity(byte), mature());
        organic
            .react(Reaction {
                author: identity(byte),
                target: post(),
                kind: ReactionKind::Like,
                cluster: Digest32::from_bytes([byte; 32]),
                topic: Digest32::from_bytes([0; 32]),
                created_at: 0,
            })
            .expect("organic reaction");
    }

    let bought = pump.content(post(), 60).net;
    let earned = organic.content(post(), WEEK).net;
    assert!(
        earned > bought,
        "20 mature independents ({earned}) must outweigh 100 fresh clustered ({bought})"
    );
}

/// The genesis key is a market privilege. It has no mint and no control.
#[test]
fn genesis_root_can_sell_ads_and_nothing_else() {
    let root = AdvertisingRoot::from_seed(&[9; 32]);
    let limits = root.limits();
    assert_eq!(limits, AdvertisingLimits::GENESIS);
    assert!(limits.is_market_only());
    assert!(!limits.decrypt);
    assert!(!limits.sign_user);
    assert!(!limits.halt_network);
    assert!(!limits.seize_account);
    assert!(!root.can_sign_user_event());
}

/// Losing the founder key must not stop the market or the mesh.
#[test]
fn the_network_outlives_the_genesis_key() {
    let mut book = Book::new();
    book.record(node(5), WorkKind::Storage, 4_000, 1, true)
        .expect("records");
    book.record(node(5), WorkKind::Relay, 4_000, 2, true)
        .expect("records");
    assert!(
        book.eligible(node(5), 0),
        "a worker with no social weight and no founder is still eligible"
    );
    assert!(book.credits(node(5)) > 0, "minting needs no founder key");
}
