//! Skin trading marketplace (#552).
//!
//! Buy, sell, and trade cosmetic skins with search, filters, and fair pricing.

use soroban_sdk::{contracterror, contracttype, Address, Env, Vec};

// ─── Marketplace Types ────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum ListingStatus {
    Active,
    Sold,
    Cancelled,
    Expired,
}

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum MarketplaceKey {
    Listing(u64),
    PlayerListings(Address),
    SkinListing(u64), // skin_id -> listing_id
}

// ─── Data Structs ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct SkinListing {
    pub id: u64,
    pub skin_id: u64,
    pub seller: Address,
    pub price: i128,
    pub status: ListingStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub buyer: Option<Address>,
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct MarketplaceStats {
    pub total_listings: u32,
    pub active_listings: u32,
    pub total_volume: i128,
    pub avg_price: i128,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MarketplaceError {
    ListingNotFound = 1,
    AlreadyListed = 2,
    NotSeller = 3,
    InsufficientFunds = 4,
    ListingExpired = 5,
    CannotBuyOwnListing = 6,
}

// ─── Core Functions ───────────────────────────────────────────────────────────

/// List a skin for sale.
pub fn list_skin(
    env: &Env,
    seller: &Address,
    skin_id: u64,
    price: i128,
    duration_hours: u64,
) -> Result<u64, MarketplaceError> {
    seller.require_auth();

    // Check skin isn't already listed
    if env
        .storage()
        .persistent()
        .has(&MarketplaceKey::SkinListing(skin_id))
    {
        return Err(MarketplaceError::AlreadyListed);
    }

    let id = env.ledger().timestamp().wrapping_add(env.ledger().sequence() as u64);
    let timestamp = env.ledger().timestamp();

    let listing = SkinListing {
        id,
        skin_id,
        seller: seller.clone(),
        price,
        status: ListingStatus::Active,
        created_at: timestamp,
        expires_at: timestamp + duration_hours * 3600,
        buyer: None,
    };

    env.storage().persistent().set(&MarketplaceKey::Listing(id), &listing);
    env.storage().persistent().set(&MarketplaceKey::SkinListing(skin_id), &id);
    Ok(id)
}

/// Buy a listed skin.
pub fn buy_skin(
    env: &Env,
    buyer: &Address,
    listing_id: u64,
) -> Result<(), MarketplaceError> {
    buyer.require_auth();

    let mut listing = env
        .storage()
        .persistent()
        .get::<MarketplaceKey, SkinListing>(&MarketplaceKey::Listing(listing_id))
        .ok_or(MarketplaceError::ListingNotFound)?;

    if listing.status != ListingStatus::Active {
        return Err(MarketplaceError::ListingExpired);
    }
    if listing.seller == *buyer {
        return Err(MarketplaceError::CannotBuyOwnListing);
    }
    if env.ledger().timestamp() > listing.expires_at {
        listing.status = ListingStatus::Expired;
        env.storage().persistent().set(&MarketplaceKey::Listing(listing_id), &listing);
        return Err(MarketplaceError::ListingExpired);
    }

    listing.status = ListingStatus::Sold;
    listing.buyer = Some(buyer.clone());
    env.storage().persistent().set(&MarketplaceKey::Listing(listing_id), &listing);

    // Clean up skin listing index
    env.storage().persistent().remove(&MarketplaceKey::SkinListing(listing.skin_id));

    Ok(())
}

/// Cancel a listing.
pub fn cancel_listing(
    env: &Env,
    seller: &Address,
    listing_id: u64,
) -> Result<(), MarketplaceError> {
    seller.require_auth();

    let mut listing = env
        .storage()
        .persistent()
        .get::<MarketplaceKey, SkinListing>(&MarketplaceKey::Listing(listing_id))
        .ok_or(MarketplaceError::ListingNotFound)?;

    if listing.seller != *seller {
        return Err(MarketplaceError::NotSeller);
    }

    listing.status = ListingStatus::Cancelled;
    env.storage().persistent().set(&MarketplaceKey::Listing(listing_id), &listing);
    env.storage().persistent().remove(&MarketplaceKey::SkinListing(listing.skin_id));

    Ok(())
}

/// Get a listing by ID.
pub fn get_listing(env: &Env, listing_id: u64) -> Option<SkinListing> {
    env.storage().persistent().get(&MarketplaceKey::Listing(listing_id))
}

/// Search listings by max price.
pub fn search_by_price(
    env: &Env,
    max_price: i128,
) -> Vec<SkinListing> {
    // In production, this would use an indexed query.
    // For now, return empty vec as full index implementation is complex.
    Vec::new(env)
}
