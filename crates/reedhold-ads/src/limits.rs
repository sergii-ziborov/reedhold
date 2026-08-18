//! What the genesis advertising token is allowed to do.

/// Fixed capability mask. These fields are not configuration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct AdvertisingLimits {
    /// May issue a short-lived ad-operator certificate.
    pub issue_operator: bool,
    /// May decrypt user messages.
    pub decrypt: bool,
    /// May sign an event as a user.
    pub sign_user: bool,
    /// May halt or rewrite the mesh.
    pub halt_network: bool,
    /// May mint identity or revoke an account.
    pub seize_account: bool,
}

impl AdvertisingLimits {
    /// Protocol-hard bounds for the genesis advertising root.
    pub const GENESIS: Self = Self {
        issue_operator: true,
        decrypt: false,
        sign_user: false,
        halt_network: false,
        seize_account: false,
    };

    /// True only if this is a market privilege, not control.
    #[must_use]
    pub const fn is_market_only(self) -> bool {
        self.issue_operator
            && !self.decrypt
            && !self.sign_user
            && !self.halt_network
            && !self.seize_account
    }
}

#[cfg(test)]
mod tests {
    use super::AdvertisingLimits;

    #[test]
    fn genesis_token_cannot_run_the_network() {
        let limits = AdvertisingLimits::GENESIS;
        assert!(limits.is_market_only());
        assert!(!limits.halt_network);
    }
}
