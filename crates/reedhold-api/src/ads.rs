//! Host view of the genesis advertising token.

use reedhold_ads::AdvertisingLimits;
use serde::Serialize;

/// What the company token may and may not do.
#[derive(Clone, Debug, Serialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct AdvertisingLimitsView {
    /// May issue a short-lived ad operator certificate.
    pub issue_operator: bool,
    /// May decrypt chats. Always false.
    pub decrypt: bool,
    /// May impersonate a user. Always false.
    pub sign_user: bool,
    /// May halt the mesh. Always false.
    pub halt_network: bool,
    /// May seize an account. Always false.
    pub seize_account: bool,
    /// Convenience: market privilege only.
    pub market_only: bool,
}

/// Protocol-hard advertising limits. No unlocked session required.
#[must_use]
pub fn advertising_limits() -> AdvertisingLimitsView {
    let limits = AdvertisingLimits::GENESIS;
    AdvertisingLimitsView {
        issue_operator: limits.issue_operator,
        decrypt: limits.decrypt,
        sign_user: limits.sign_user,
        halt_network: limits.halt_network,
        seize_account: limits.seize_account,
        market_only: limits.is_market_only(),
    }
}

#[cfg(test)]
mod tests {
    use super::advertising_limits;

    #[test]
    fn token_is_ads_only() {
        let view = advertising_limits();
        assert!(view.market_only);
        assert!(!view.halt_network);
    }
}
