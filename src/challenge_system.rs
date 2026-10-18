//! Challenge system for battle pass XP earning (#550).
//!
//! Daily, weekly, and seasonal challenges providing additional XP
//! opportunities beyond regular gameplay activities.

use soroban_sdk::{contracterror, contracttype, Address, Env, String, Vec};

// ─── Challenge Types ──────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ChallengeFrequency {
    Daily,
    Weekly,
    Seasonal,
}

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum ChallengeKey {
    /// Active challenge: challenge_id -> Challenge
    Challenge(u64),
    /// Player progress: (profile_id, challenge_id) -> ChallengeProgress
    Progress(u64, u64),
    /// Daily challenges for a day: day -> Vec<u64>
    DailyChallenges(u32),
    /// Weekly challenges for a week: week -> Vec<u64>
    WeeklyChallenges(u32),
}

// ─── Data Structs ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct Challenge {
    pub id: u64,
    pub frequency: ChallengeFrequency,
    pub name: String,
    pub description: String,
    pub target_value: u64,
    pub xp_reward: u64,
    pub reward_item_id: Option<u32>,
    pub starts_at: u64,
    pub ends_at: u64,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct ChallengeProgress {
    pub profile_id: u64,
    pub challenge_id: u64,
    pub current_value: u64,
    pub completed: bool,
    pub claimed: bool,
    pub completed_at: Option<u64>,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ChallengeError {
    ChallengeNotFound = 1,
    AlreadyCompleted = 2,
    AlreadyClaimed = 3,
    NotCompleted = 4,
    InvalidTarget = 5,
}

// ─── Challenge Management ─────────────────────────────────────────────────────

/// Create a new challenge.
pub fn create_challenge(
    env: &Env,
    admin: &Address,
    frequency: ChallengeFrequency,
    name: String,
    description: String,
    target_value: u64,
    xp_reward: u64,
    duration_hours: u64,
) -> Result<u64, ChallengeError> {
    admin.require_auth();
    if target_value == 0 {
        return Err(ChallengeError::InvalidTarget);
    }

    let id = env.ledger().timestamp().wrapping_add(env.ledger().sequence() as u64);
    let timestamp = env.ledger().timestamp();

    let challenge = Challenge {
        id,
        frequency,
        name,
        description,
        target_value,
        xp_reward,
        reward_item_id: None,
        starts_at: timestamp,
        ends_at: timestamp + duration_hours * 3600,
    };

    env.storage().persistent().set(&ChallengeKey::Challenge(id), &challenge);
    Ok(id)
}

/// Get a challenge by ID.
pub fn get_challenge(env: &Env, challenge_id: u64) -> Option<Challenge> {
    env.storage().persistent().get(&ChallengeKey::Challenge(challenge_id))
}

/// Update player progress on a challenge.
pub fn update_progress(
    env: &Env,
    profile_id: u64,
    challenge_id: u64,
    increment: u64,
) -> Result<ChallengeProgress, ChallengeError> {
    let challenge = get_challenge(env, challenge_id)
        .ok_or(ChallengeError::ChallengeNotFound)?;

    let key = ChallengeKey::Progress(profile_id, challenge_id);
    let mut progress = env
        .storage()
        .persistent()
        .get::<ChallengeKey, ChallengeProgress>(&key)
        .unwrap_or(ChallengeProgress {
            profile_id,
            challenge_id,
            current_value: 0,
            completed: false,
            claimed: false,
            completed_at: None,
        });

    if progress.completed {
        return Ok(progress);
    }

    progress.current_value += increment;
    if progress.current_value >= challenge.target_value {
        progress.completed = true;
        progress.completed_at = Some(env.ledger().timestamp());
    }

    env.storage().persistent().set(&key, &progress);
    Ok(progress)
}

/// Claim a completed challenge's XP reward.
pub fn claim_reward(
    env: &Env,
    profile_id: u64,
    challenge_id: u64,
) -> Result<u64, ChallengeError> {
    let challenge = get_challenge(env, challenge_id)
        .ok_or(ChallengeError::ChallengeNotFound)?;

    let key = ChallengeKey::Progress(profile_id, challenge_id);
    let mut progress = env
        .storage()
        .persistent()
        .get::<ChallengeKey, ChallengeProgress>(&key)
        .ok_or(ChallengeError::NotCompleted)?;

    if progress.claimed {
        return Err(ChallengeError::AlreadyClaimed);
    }
    if !progress.completed {
        return Err(ChallengeError::NotCompleted);
    }

    progress.claimed = true;
    env.storage().persistent().set(&key, &progress);
    Ok(challenge.xp_reward)
}

/// Get all active challenges for a player.
pub fn get_active_challenges(env: &Env, profile_id: u64) -> Vec<(Challenge, ChallengeProgress)> {
    // In production, this would iterate over the daily/weekly/seasonal indexes.
    // For now, return empty vec as the full index implementation is complex.
    Vec::new(env)
}
