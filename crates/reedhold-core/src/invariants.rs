//! Protocol invariants that later stages must not quietly drop.

/// Named invariants. Hosts and the MCP server expose this list.
pub const INVARIANTS: &[&str] = &[
    "password_does_not_create_identity",
    "company_is_not_source_of_truth",
    "consumer_node_is_bounded",
    "no_network_master_key",
    "chain_does_not_store_private_bytes",
    "phone_is_not_a_durable_seeder",
    "company_host_is_optional",
    "rotating_relays_are_not_a_fixed_set",
    "blocking_relays_does_not_halt_the_network",
    "advertising_token_is_not_network_control",
];
