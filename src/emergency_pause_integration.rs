//! Emergency pause integration helpers for all contract modules.
//!
//! This module provides decorators and helpers to integrate emergency pause checks
//! into sensitive operations across the contract suite.

use crate::emergency_controls;
use soroban_sdk::{Env, Symbol, symbol_short};

/// Check if contract is paused and fail if so.
/// Used as a guard at the start of sensitive operations.
pub fn require_not_paused(env: &Env) -> Result<(), emergency_controls::EmergencyError> {
    emergency_controls::require_not_paused(env)
}

/// Macro-like helper: combine pause check with operation.
/// Usage: check_pause_then_execute(env, || { operation_code })
pub fn check_then_execute<F, T>(
    env: &Env,
    operation: F,
) -> Result<T, emergency_controls::EmergencyError>
where
    F: FnOnce() -> T,
{
    require_not_paused(env)?;
    Ok(operation())
}

/// List of critical operations that must check pause state.
pub struct CriticalOperations;

impl CriticalOperations {
    /// Operations that should check pause state
    pub fn requires_pause_check(operation: &str) -> bool {
        matches!(
            operation,
            // Access Control
            "grant_role"
                | "revoke_role"
                | "transfer_admin"
                | "grant_permission"
                | "revoke_permission"
                // Ship Operations
                | "transfer_ship"
                | "apply_upgrade"
                | "batch_upgrade"
                // Resource Operations
                | "mint_resource"
                | "burn_resource"
                | "transfer_resource"
                // Trading
                | "list_resource"
                | "buy_offer"
                | "sell_offer"
                // Fractional Resources
                | "fractionalize_resource"
                | "merge_fractions"
                | "transfer_share"
                // Staking
                | "stake_resource"
                | "unstake_resource"
                | "claim_rewards"
        )
    }

    /// Operations that should NOT check pause state (read-only)
    pub fn is_read_only(operation: &str) -> bool {
        matches!(
            operation,
            "get_balance"
                | "get_holdings"
                | "get_leaderboard"
                | "get_audit_log"
                | "get_economy_metrics"
                | "query_prices"
                | "check_permission"
                | "get_role"
        )
    }
}

/// Audit log entry for emergency pause events.
pub fn log_pause_event(env: &Env, admin: &soroban_sdk::Address) -> Result<(), crate::audit_logger::AuditLoggerError> {
    use crate::audit_integration;
    let action = symbol_short!("paused");
    let details = soroban_sdk::BytesN::from_slice(env, "emergency_pause".as_bytes());
    crate::audit_logger::log_audit_event(env, Some(admin), action, details)
}

/// Audit log entry for emergency unpause events.
pub fn log_unpause_event(env: &Env, admin: &soroban_sdk::Address) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("unpaused");
    let details = soroban_sdk::BytesN::from_slice(env, "emergency_unpause".as_bytes());
    crate::audit_logger::log_audit_event(env, Some(admin), action, details)
}

/// Pre-flight checks for sensitive operations.
pub struct PreFlightChecks;

impl PreFlightChecks {
    /// Run all pre-flight checks before executing sensitive operation.
    /// Returns Ok(()) if all checks pass, Err if any check fails.
    pub fn run_sensitive_operation_checks(
        env: &Env,
        caller: &soroban_sdk::Address,
        operation: &str,
    ) -> Result<(), emergency_controls::EmergencyError> {
        // Check 1: Contract not paused
        require_not_paused(env)?;

        // Check 2: Operation requires pause check
        if !CriticalOperations::requires_pause_check(operation) && !CriticalOperations::is_read_only(operation) {
            // Operation not in either list - be safe and require pause check
            require_not_paused(env)?;
        }

        Ok(())
    }
}

/// Integration points for emergency pause across modules.
pub const PAUSE_CHECK_LOCATIONS: &[&str] = &[
    "src/access_control.rs - grant_role, revoke_role, transfer_admin",
    "src/ship_upgrade.rs - apply_upgrade, batch_upgrade",
    "src/dex_integration.rs - list_resource, buy_offer, sell_offer",
    "src/fractional_resources.rs - fractionalize_resource, merge_fractions",
    "src/staking.rs - stake_resource, unstake_resource, claim_rewards",
    "src/ship_nft.rs - transfer_ship, mint_ship",
    "src/badges.rs - transfer_badge, mint_badge",
    "src/treasury_vault.rs - deposit, withdraw, claim_treasure",
];
