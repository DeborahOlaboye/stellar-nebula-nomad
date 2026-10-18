//! Asteroid Racing mini-game (#551).
//!
//! Physics-based racing through procedurally generated asteroid fields.

use soroban_sdk::{contracterror, contracttype, Address, Env, Vec};

// ─── Game State ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum RaceState {
    Lobby,
    Countdown,
    Racing,
    Finished,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct RaceCourse {
    pub id: u64,
    pub name: soroban_sdk::String,
    pub difficulty: u32,     // 1-10
    pub asteroid_count: u32,
    pub checkpoint_count: u32,
    pub best_time_ms: Option<u64>,
    pub best_player: Option<Address>,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct RaceSession {
    pub id: u64,
    pub course_id: u64,
    pub players: Vec<Address>,
    pub state: RaceState,
    pub started_at: Option<u64>,
    pub finish_times: Vec<(Address, u64)>,
    pub max_players: u32,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum RaceError {
    RaceFull = 1,
    RaceNotFound = 2,
    AlreadyRacing = 3,
    InvalidCourse = 4,
}

// ─── Core Functions ───────────────────────────────────────────────────────────

/// Create a new race course.
pub fn create_course(
    env: &Env,
    admin: &Address,
    name: soroban_sdk::String,
    difficulty: u32,
    asteroid_count: u32,
    checkpoint_count: u32,
) -> u64 {
    admin.require_auth();
    let id = env.ledger().timestamp().wrapping_add(env.ledger().sequence() as u64);

    let course = RaceCourse {
        id,
        name,
        difficulty,
        asteroid_count,
        checkpoint_count,
        best_time_ms: None,
        best_player: None,
    };

    env.storage().persistent().set(&RaceKey::Course(id), &course);
    id
}

#[derive(Clone)]
#[contracttype]
pub enum RaceKey {
    Course(u64),
    Session(u64),
}

/// Create a race session.
pub fn create_race(
    env: &Env,
    creator: &Address,
    course_id: u64,
    max_players: u32,
) -> Result<u64, RaceError> {
    creator.require_auth();
    let course = env
        .storage()
        .persistent()
        .get::<RaceKey, RaceCourse>(&RaceKey::Course(course_id))
        .ok_or(RaceError::InvalidCourse)?;

    let id = env.ledger().timestamp().wrapping_add(env.ledger().sequence() as u64);
    let mut players = Vec::new(env);
    players.push_back(creator.clone());

    let session = RaceSession {
        id,
        course_id: course.id,
        players,
        state: RaceState::Lobby,
        started_at: None,
        finish_times: Vec::new(env),
        max_players,
    };

    env.storage().persistent().set(&RaceKey::Session(id), &session);
    Ok(id)
}

/// Join a race session.
pub fn join_race(env: &Env, player: &Address, race_id: u64) -> Result<(), RaceError> {
    player.require_auth();
    let mut session = env
        .storage()
        .persistent()
        .get::<RaceKey, RaceSession>(&RaceKey::Session(race_id))
        .ok_or(RaceError::RaceNotFound)?;

    if session.players.len() >= session.max_players {
        return Err(RaceError::RaceFull);
    }
    if session.state != RaceState::Lobby {
        return Err(RaceError::AlreadyRacing);
    }

    session.players.push_back(player.clone());
    env.storage().persistent().set(&RaceKey::Session(race_id), &session);
    Ok(())
}

/// Record a player's finish time.
pub fn record_finish(
    env: &Env,
    race_id: u64,
    player: &Address,
    time_ms: u64,
) -> Result<(), RaceError> {
    let mut session = env
        .storage()
        .persistent()
        .get::<RaceKey, RaceSession>(&RaceKey::Session(race_id))
        .ok_or(RaceError::RaceNotFound)?;

    session.finish_times.push_back((player.clone(), time_ms));

    // Update course best time
    if let Some(mut course) = env
        .storage()
        .persistent()
        .get::<RaceKey, RaceCourse>(&RaceKey::Course(session.course_id))
    {
        if course.best_time_ms.is_none() || time_ms < course.best_time_ms.unwrap() {
            course.best_time_ms = Some(time_ms);
            course.best_player = Some(player.clone());
            env.storage().persistent().set(&RaceKey::Course(session.course_id), &course);
        }
    }

    env.storage().persistent().set(&RaceKey::Session(race_id), &session);
    Ok(())
}
