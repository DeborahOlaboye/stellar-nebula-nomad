//! Space Poker mini-game (#551).
//!
//! Texas Hold'em rules with cosmic theming, AI opponents, and multiplayer support.

use soroban_sdk::{contracterror, contracttype, Address, Env, Vec};

// ─── Card Types ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum Suit {
    Stars,
    Nebulae,
    Planets,
    Comets,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum Rank {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
    pub is_cosmic: bool, // Supernova, black hole, wormhole wildcards
}

// ─── Game State ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum PokerState {
    WaitingForPlayers,
    Preflop,
    Flop,
    Turn,
    River,
    Showdown,
    Finished,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct PokerGame {
    pub id: u64,
    pub players: Vec<Address>,
    pub pot: i128,
    pub community_cards: Vec<Card>,
    pub state: PokerState,
    pub current_player: u32,
    pub min_buy_in: i128,
    pub max_players: u32,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum PokerError {
    GameFull = 1,
    NotYourTurn = 2,
    InvalidAction = 3,
    InsufficientChips = 4,
    GameNotFound = 5,
}

// ─── Core Functions ───────────────────────────────────────────────────────────

/// Create a new poker game.
pub fn create_game(
    env: &Env,
    creator: &Address,
    min_buy_in: i128,
    max_players: u32,
) -> Result<u64, PokerError> {
    creator.require_auth();
    let id = env.ledger().timestamp().wrapping_add(env.ledger().sequence() as u64);

    let mut players = Vec::new(env);
    players.push_back(creator.clone());

    let game = PokerGame {
        id,
        players,
        pot: 0,
        community_cards: Vec::new(env),
        state: PokerState::WaitingForPlayers,
        current_player: 0,
        min_buy_in,
        max_players,
    };

    env.storage().persistent().set(&PokerKey::Game(id), &game);
    Ok(id)
}

#[derive(Clone)]
#[contracttype]
pub enum PokerKey {
    Game(u64),
    PlayerHand(u64, Address),
}

/// Join a poker game.
pub fn join_game(env: &Env, player: &Address, game_id: u64) -> Result<(), PokerError> {
    player.require_auth();
    let mut game = env
        .storage()
        .persistent()
        .get::<PokerKey, PokerGame>(&PokerKey::Game(game_id))
        .ok_or(PokerError::GameNotFound)?;

    if game.players.len() >= game.max_players {
        return Err(PokerError::GameFull);
    }

    game.players.push_back(player.clone());
    env.storage().persistent().set(&PokerKey::Game(game_id), &game);
    Ok(())
}

/// Evaluate a hand (simplified — returns a score).
pub fn evaluate_hand(cards: &Vec<Card>) -> u32 {
    let mut score = 0u32;
    for card in cards.iter() {
        let rank_score = match card.rank {
            Rank::Ace => 14,
            Rank::King => 13,
            Rank::Queen => 12,
            Rank::Jack => 11,
            Rank::Ten => 10,
            Rank::Nine => 9,
            Rank::Eight => 8,
            Rank::Seven => 7,
            Rank::Six => 6,
            Rank::Five => 5,
            Rank::Four => 4,
            Rank::Three => 3,
            Rank::Two => 2,
        };
        score += rank_score;
        if card.is_cosmic {
            score += 50; // Cosmic cards are wild
        }
    }
    score
}
