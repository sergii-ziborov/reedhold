//! DMs and small groups over the in-process mesh fabric.

use crate::inbox::{CircleView, TalkView, circle_view, ingest_one};
use crate::mesh::{MeshSession, RouteView};
use crate::session::Session;
use reedhold_core::{ConversationId, Error, IdentityId, Result, decode32, encode_hex};
use reedhold_event::{EventKind, TalkBody, TalkPacket, dm_conversation, seal_message};
use reedhold_protocol::Circle;

/// Social overlay. Peer ids are identity digests.
pub struct TalkNet {
    mesh: MeshSession,
}

impl TalkNet {
    /// Same lottery as [`MeshSession::open`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when a hex id is invalid.
    pub fn open(
        epoch: u64,
        prior_commit_hex: &str,
        candidate_hexes: &[String],
        company_hex: Option<&str>,
        relay_count: Option<u16>,
    ) -> Result<Self> {
        Ok(Self {
            mesh: MeshSession::open(
                epoch,
                prior_commit_hex,
                candidate_hexes,
                company_hex,
                relay_count,
            )?,
        })
    }

    /// Bring a talk peer online.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn online(&mut self, peer_hex: &str) -> Result<()> {
        self.mesh.online(peer_hex)
    }

    /// Take a talk peer offline.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn offline(&mut self, peer_hex: &str) -> Result<()> {
        self.mesh.offline(peer_hex)
    }

    /// Block a host. Talk keeps running.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Codec`] when the hex id is invalid.
    pub fn block(&mut self, peer_hex: &str) -> Result<()> {
        self.mesh.block(peer_hex)
    }

    /// Seal a DM under the pairwise key and send it over the fabric.
    ///
    /// # Errors
    ///
    /// Returns a protocol error when keys or hex ids are wrong.
    pub fn dm(
        &mut self,
        from: &mut Session,
        to_hex: &str,
        to_msg_pub_hex: &str,
        text: &str,
    ) -> Result<RouteView> {
        let to = IdentityId::from_hex(to_hex)?;
        let key = from.account.messaging().agree(
            &decode32(to_msg_pub_hex)?,
            from.account.identity(),
            to,
        )?;
        let body = TalkBody {
            conversation: dm_conversation(from.account.identity(), to),
            envelope: seal_message(&key, text.as_bytes())?,
        };
        self.dispatch(from, to_hex, EventKind::DirectMessage, &body)
    }

    /// Create a small group. The epoch key stays on this session.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Entropy`] when a group id cannot be drawn.
    pub fn create_circle(&self, owner: &mut Session, name: &str) -> Result<CircleView> {
        let circle = Circle::create(owner.account.identity(), name)?;
        let view = circle_view(&circle);
        let meta = circle.invite_body().encode()?;
        let event = owner.account.emit(EventKind::GroupCreate, &meta)?;
        owner.push_log(&event, &meta)?;
        owner.remember_circle(circle);
        Ok(view)
    }

    /// Wrap the group key for `member` and send an invite.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the group is unknown.
    pub fn invite(
        &mut self,
        owner: &mut Session,
        group_hex: &str,
        member_hex: &str,
        member_msg_pub_hex: &str,
    ) -> Result<RouteView> {
        let group = ConversationId::from_hex(group_hex)?;
        let member = IdentityId::from_hex(member_hex)?;
        owner.circle_mut(group)?.include(member);
        let invite = owner.circle(group)?.invite_body();
        let wrap = owner.account.messaging().agree(
            &decode32(member_msg_pub_hex)?,
            owner.account.identity(),
            member,
        )?;
        let body = TalkBody {
            conversation: group,
            envelope: seal_message(&wrap, &invite.encode()?)?,
        };
        self.dispatch(owner, member_hex, EventKind::GroupInvite, &body)
    }

    /// Fan the same sealed group message out to every other member.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Event`] when the sender is not in the group.
    pub fn send_circle(
        &mut self,
        from: &mut Session,
        group_hex: &str,
        text: &str,
    ) -> Result<Vec<RouteView>> {
        let group = ConversationId::from_hex(group_hex)?;
        let me = from.account.identity();
        if !from.circle(group)?.members.contains(&me) {
            return Err(Error::Event("not a group member"));
        }
        let sealed = from.circle(group)?.seal(text.as_bytes())?;
        let body = TalkBody {
            conversation: group,
            envelope: sealed,
        };
        let members: Vec<IdentityId> = from
            .circle(group)?
            .members
            .iter()
            .copied()
            .filter(|id| *id != me)
            .collect();
        let mut routes = Vec::with_capacity(members.len());
        for member in members {
            routes.push(self.dispatch(from, &member.to_hex(), EventKind::GroupMessage, &body)?);
        }
        Ok(routes)
    }

    /// Drain and decrypt talk for this session.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Mesh`] when the peer is unknown.
    pub fn inbox(&mut self, session: &mut Session) -> Result<Vec<TalkView>> {
        let mut out = Vec::new();
        for item in self.mesh.drain(&session.peer_hex())? {
            if let Ok(view) = ingest_one(session, &item) {
                out.push(view);
            }
        }
        Ok(out)
    }

    fn dispatch(
        &mut self,
        from: &mut Session,
        to_hex: &str,
        kind: EventKind,
        body: &TalkBody,
    ) -> Result<RouteView> {
        let encoded = body.encode()?;
        let event = from.account.emit(kind, &encoded)?;
        from.push_log(&event, &encoded)?;
        let packet = TalkPacket {
            author: from.account.identity(),
            messaging_public: from.account.messaging().public_bytes(),
            device_public: from.account.device_public(),
            event: event.encode()?,
            body: encoded,
        };
        self.mesh
            .send(&from.peer_hex(), to_hex, &encode_hex(&packet.encode()?))
    }
}
