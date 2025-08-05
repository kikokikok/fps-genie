use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use cs2_common::BehavioralVector;

/// Match metadata stored in relational database
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Match {
    pub id: Uuid,
    pub match_id: String,
    pub tournament: Option<String>,
    pub map_name: String,
    pub team1: String,
    pub team2: String,
    pub score_team1: i32,
    pub score_team2: i32,
    pub demo_file_path: String,
    pub demo_file_size: i64,
    pub tick_rate: i32,
    pub duration_seconds: i32,
    pub created_at: DateTime<Utc>,
    pub processed_at: Option<DateTime<Utc>>,
    pub processing_status: ProcessingStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "processing_status", rename_all = "lowercase")]
pub enum ProcessingStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

/// Player information
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Player {
    pub id: Uuid,
    pub steamid: i64,
    pub name: String,
    pub team: Option<String>,
    pub is_professional: bool,
    pub rating: Option<f32>,
    pub created_at: DateTime<Utc>,
}

/// Match participation linking players to matches
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MatchParticipation {
    pub id: Uuid,
    pub match_id: Uuid,
    pub player_id: Uuid,
    pub team_side: String, // "T" or "CT"
    pub final_score: i32,
    pub kills: i32,
    pub deaths: i32,
    pub assists: i32,
    pub adr: f32, // Average Damage per Round
    pub rating: f32,
}

/// Time-series player snapshot for TimescaleDB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerSnapshot {
    pub timestamp: DateTime<Utc>,
    pub match_id: Uuid,
    pub tick: u32,
    pub steamid: i64,
    pub round_number: i32,
    pub health: f32,
    pub armor: f32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub pos_z: f32,
    pub vel_x: f32,
    pub vel_y: f32,
    pub vel_z: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub weapon_id: u16,
    pub ammo_clip: i32,
    pub ammo_reserve: i32,
    pub is_alive: bool,
    pub is_airborne: bool,
    pub is_scoped: bool,
    pub is_walking: bool,
    pub flash_duration: f32,
    pub money: i32,
    pub equipment_value: i32,
}

/// Key moment metadata for behavioral analysis
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct KeyMoment {
    pub id: Uuid,
    pub match_id: Uuid,
    pub moment_type: KeyMomentType,
    pub start_tick: u32,
    pub end_tick: u32,
    pub players_involved: Vec<i64>, // steamids
    pub outcome: String,
    pub importance_score: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "key_moment_type", rename_all = "lowercase")]
pub enum KeyMomentType {
    Clutch,
    Ace,
    ImportantDuel,
    EcoRound,
    ForceBuy,
    Retake,
    Execute,
    Flank,
}

/// Vector embeddings for similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehavioralEmbedding {
    pub id: String,
    pub match_id: String,
    pub moment_id: String,
    pub player_steamid: i64,
    pub moment_type: String,
    pub vector: Vec<f32>, // High-dimensional behavioral representation
    pub metadata: serde_json::Value,
}

impl From<BehavioralVector> for PlayerSnapshot {
    fn from(bv: BehavioralVector) -> Self {
        PlayerSnapshot {
            timestamp: Utc::now(), // Will be set properly during processing
            match_id: Uuid::new_v4(), // Will be set properly during processing
            tick: bv.tick,
            steamid: bv.steamid as i64,
            round_number: 0, // Will be calculated during processing
            health: bv.health,
            armor: bv.armor,
            pos_x: bv.pos_x,
            pos_y: bv.pos_y,
            pos_z: bv.pos_z,
            vel_x: bv.vel_x,
            vel_y: bv.vel_y,
            vel_z: bv.vel_z,
            yaw: bv.yaw,
            pitch: bv.pitch,
            weapon_id: bv.weapon_id,
            ammo_clip: bv.ammo as i32,
            ammo_reserve: 0, // Not available in BehavioralVector
            is_alive: true, // Will be determined during processing
            is_airborne: bv.is_airborne > 0.5,
            is_scoped: false, // Not available in BehavioralVector
            is_walking: false, // Not available in BehavioralVector
            flash_duration: 0.0, // Not available in BehavioralVector
            money: 0, // Will be extracted during processing
            equipment_value: 0, // Will be calculated during processing
        }
    }
}
