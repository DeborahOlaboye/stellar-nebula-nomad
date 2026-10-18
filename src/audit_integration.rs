//! Audit integration helpers for logging sensitive operations.
//!
//! This module provides convenience functions for adding audit logging to sensitive
//! operations like transfers, upgrades, and permission changes.

use crate::audit_logger;
use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol, symbol_short};

/// Log a role grant operation.
pub fn log_role_grant(
    env: &Env,
    admin: &Address,
    role: &Symbol,
    target: &Address,
    expiry: Option<u32>,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("role_grant");
    let mut details_vec = role.to_string().into_bytes();
    details_vec.extend_from_slice(target.to_string().as_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(admin), action, details)?;
    Ok(())
}

/// Log a role revocation operation.
pub fn log_role_revoke(
    env: &Env,
    admin: &Address,
    role: &Symbol,
    target: &Address,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("role_rev");
    let mut details_vec = role.to_string().into_bytes();
    details_vec.extend_from_slice(target.to_string().as_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(admin), action, details)?;
    Ok(())
}

/// Log a permission grant operation.
pub fn log_permission_grant(
    env: &Env,
    admin: &Address,
    role: &Symbol,
    action: &Symbol,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let log_action = symbol_short!("perm_gran");
    let mut details_vec = role.to_string().into_bytes();
    details_vec.extend_from_slice(action.to_string().as_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(admin), log_action, details)?;
    Ok(())
}

/// Log a transfer operation.
pub fn log_transfer(
    env: &Env,
    from: &Address,
    to: &Address,
    amount: i128,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("transfer");
    let mut details_vec = from.to_string().into_bytes();
    details_vec.extend_from_slice(to.to_string().as_bytes());
    details_vec.extend_from_slice(&amount.to_be_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(from), action, details)?;
    Ok(())
}

/// Log a ship upgrade operation.
pub fn log_ship_upgrade(
    env: &Env,
    player: &Address,
    ship_id: u64,
    upgrade_level: u32,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("upgrade");
    let mut details_vec = ship_id.to_be_bytes().to_vec();
    details_vec.extend_from_slice(&upgrade_level.to_be_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(player), action, details)?;
    Ok(())
}

/// Log a fractional resource operation.
pub fn log_fractional_operation(
    env: &Env,
    player: &Address,
    operation: &str,
    share_count: u32,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = if operation == "fractionalize" {
        symbol_short!("frac_op")
    } else {
        symbol_short!("merge_op")
    };
    let mut details_vec = operation.as_bytes().to_vec();
    details_vec.extend_from_slice(&share_count.to_be_bytes());

    let details = BytesN::from_slice(env, &details_vec);
    audit_logger::log_audit_event(env, Some(player), action, details)?;
    Ok(())
}

/// Log an admin operation.
pub fn log_admin_operation(
    env: &Env,
    admin: &Address,
    operation: &str,
) -> Result<(), crate::audit_logger::AuditLoggerError> {
    let action = symbol_short!("admin_op");
    let details = BytesN::from_slice(env, operation.as_bytes());
    audit_logger::log_audit_event(env, Some(admin), action, details)?;
    Ok(())
}
