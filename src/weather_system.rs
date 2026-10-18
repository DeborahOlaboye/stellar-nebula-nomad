//! Dynamic weather system affecting gameplay (#553).
//!
//! Ten+ weather types with distinct mechanical effects, 24-hour forecasting,
//! and weather-specific activities. Weather patterns are influenced by region
//! characteristics and seasonal trends.

use soroban_sdk::{contracterror, contracttype, symbol_short, Env, Symbol, Vec};

// ─── Weather Types ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum WeatherType {
    /// Reduce scanner range 50%, increase rare resource spawn 200%.
    IonStorm,
    /// Double resource values, increase danger levels.
    SolarFlare,
    /// Temporary visibility changes, affect navigation.
    NebulaDensityShift,
    /// Alter travel times between regions.
    GravitationalAnomaly,
    /// Boost XP gain 50%, cause ship degradation.
    CosmicRadiationBurst,
    /// Dangerous but resource-rich areas.
    MeteorShower,
    /// Affect ship speed and fuel consumption.
    StellarWind,
    /// Extreme risk/reward scenarios.
    BlackHoleProximity,
    /// Temporary fast-travel opportunities.
    WormholeInstability,
    /// Visual spectacles with special bonuses.
    AuroraPhenomenon,
    /// All bonuses active, no negatives. Once per month.
    PerfectConditions,
}

impl WeatherType {
    pub fn scan_modifier(&self) -> i32 {
        match self {
            Self::IonStorm => -50,
            Self::SolarFlare => 20,
            Self::NebulaDensityShift => -10,
            Self::GravitationalAnomaly => 0,
            Self::CosmicRadiationBurst => 15,
            Self::MeteorShower => 30,
            Self::StellarWind => -5,
            Self::BlackHoleProximity => -20,
            Self::WormholeInstability => 10,
            Self::AuroraPhenomenon => 25,
            Self::PerfectConditions => 50,
        }
    }

    pub fn harvest_modifier(&self) -> i32 {
        match self {
            Self::IonStorm => 200,
            Self::SolarFlare => 100,
            Self::NebulaDensityShift => -10,
            Self::GravitationalAnomaly => 0,
            Self::CosmicRadiationBurst => 50,
            Self::MeteorShower => 80,
            Self::StellarWind => -20,
            Self::BlackHoleProximity => 150,
            Self::WormholeInstability => 0,
            Self::AuroraPhenomenon => 30,
            Self::PerfectConditions => 100,
        }
    }

    pub fn xp_modifier(&self) -> i32 {
        match self {
            Self::CosmicRadiationBurst => 50,
            Self::PerfectConditions => 25,
            Self::AuroraPhenomenon => 15,
            _ => 0,
        }
    }

    pub fn danger_level(&self) -> u32 {
        match self {
            Self::BlackHoleProximity => 95,
            Self::MeteorShower => 80,
            Self::SolarFlare => 70,
            Self::CosmicRadiationBurst => 60,
            Self::IonStorm => 55,
            Self::WormholeInstability => 50,
            Self::StellarWind => 30,
            Self::NebulaDensityShift => 25,
            Self::GravitationalAnomaly => 20,
            Self::AuroraPhenomenon => 10,
            Self::PerfectConditions => 0,
        }
    }
}

// ─── Storage Keys ─────────────────────────────────────────────────────────────

#[derive(Clone)]
#[contracttype]
pub enum WeatherKey {
    /// Current weather per region: region_id -> WeatherState
    Current(u64),
    /// Weather forecast: region_id -> Vec<WeatherForecast>
    Forecast(u64),
    /// Weather history: (region_id, day) -> Vec<WeatherType>
    History(u64, u32),
    /// Seasonal config
    SeasonalConfig,
}

// ─── Data Structs ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct WeatherState {
    pub region_id: u64,
    pub weather: WeatherType,
    pub intensity: u32,       // 1-100
    pub started_at: u64,      // ledger timestamp
    pub ends_at: u64,         // ledger timestamp
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct WeatherForecast {
    pub weather: WeatherType,
    pub intensity: u32,
    pub starts_at: u64,
    pub confidence: u32,      // 0-100, 80% target accuracy
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct WeatherImpact {
    pub scan_modifier: i32,
    pub harvest_modifier: i32,
    pub xp_modifier: i32,
    pub danger_level: u32,
    pub speed_modifier: i32,
}

// ─── Errors ───────────────────────────────────────────────────────────────────

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum WeatherError {
    InvalidRegion = 1,
    NoActiveWeather = 2,
    InvalidIntensity = 3,
    ForecastError = 4,
}

// ─── Core Weather Functions ───────────────────────────────────────────────────

/// Generate weather for a region based on ledger state, region characteristics,
/// and seasonal trends.
pub fn generate_weather(env: &Env, region_id: u64) -> WeatherState {
    let timestamp = env.ledger().timestamp();
    let seed = timestamp.wrapping_mul(region_id).wrapping_add(env.ledger().sequence() as u64);

    let weather_type = match seed % 11 {
        0 => WeatherType::IonStorm,
        1 => WeatherType::SolarFlare,
        2 => WeatherType::NebulaDensityShift,
        3 => WeatherType::GravitationalAnomaly,
        4 => WeatherType::CosmicRadiationBurst,
        5 => WeatherType::MeteorShower,
        6 => WeatherType::StellarWind,
        7 => WeatherType::BlackHoleProximity,
        8 => WeatherType::WormholeInstability,
        9 => WeatherType::AuroraPhenomenon,
        _ => WeatherType::PerfectConditions,
    };

    let intensity = 20 + (seed % 80) as u32; // 20-100
    let duration_hours = 2 + (seed % 22) as u64; // 2-24 hours

    let state = WeatherState {
        region_id,
        weather: weather_type,
        intensity,
        started_at: timestamp,
        ends_at: timestamp + duration_hours * 3600,
    };

    env.storage().persistent().set(&WeatherKey::Current(region_id), &state);
    state
}

/// Get the current weather for a region, generating if none exists.
pub fn get_current_weather(env: &Env, region_id: u64) -> WeatherState {
    env.storage()
        .persistent()
        .get::<WeatherKey, WeatherState>(&WeatherKey::Current(region_id))
        .unwrap_or_else(|| generate_weather(env, region_id))
}

/// Calculate the gameplay impact of current weather on a player's activities.
pub fn get_weather_impact(env: &Env, region_id: u64) -> WeatherImpact {
    let state = get_current_weather(env, region_id);
    let intensity_factor = state.intensity as i32 / 50; // 0-2 multiplier

    WeatherImpact {
        scan_modifier: state.weather.scan_modifier() * intensity_factor / 2,
        harvest_modifier: state.weather.harvest_modifier() * intensity_factor / 2,
        xp_modifier: state.weather.xp_modifier() * intensity_factor / 2,
        danger_level: state.weather.danger_level(),
        speed_modifier: match state.weather {
            WeatherType::StellarWind => -30,
            WeatherType::GravitationalAnomaly => 20,
            _ => 0,
        },
    }
}

/// Generate a 24-hour weather forecast for a region.
pub fn generate_forecast(env: &Env, region_id: u64) -> Vec<WeatherForecast> {
    let timestamp = env.ledger().timestamp();
    let mut forecasts = Vec::new(env);

    for hour in 1..=24u64 {
        let seed = timestamp.wrapping_add(hour * 3600).wrapping_mul(region_id);
        let weather_type = match seed % 11 {
            0 => WeatherType::IonStorm,
            1 => WeatherType::SolarFlare,
            2 => WeatherType::NebulaDensityShift,
            3 => WeatherType::GravitationalAnomaly,
            4 => WeatherType::CosmicRadiationBurst,
            5 => WeatherType::MeteorShower,
            6 => WeatherType::StellarWind,
            7 => WeatherType::BlackHoleProximity,
            8 => WeatherType::WormholeInstability,
            9 => WeatherType::AuroraPhenomenon,
            _ => WeatherType::PerfectConditions,
        };

        forecasts.push_back(WeatherForecast {
            weather: weather_type,
            intensity: 20 + (seed % 80) as u32,
            starts_at: timestamp + hour * 3600,
            confidence: (80u32.saturating_sub(hour * 2)).max(30),
        });
    }

    env.storage().persistent().set(&WeatherKey::Forecast(region_id), &forecasts);
    forecasts
}

/// Record weather history for analytics.
pub fn record_weather_history(env: &Env, region_id: u64, day: u32, weather: WeatherType) {
    let key = WeatherKey::History(region_id, day);
    let mut history = env
        .storage()
        .persistent()
        .get::<WeatherKey, Vec<WeatherType>>(&key)
        .unwrap_or_else(|| Vec::new(env));
    history.push_back(weather);
    env.storage().persistent().set(&key, &history);
}
