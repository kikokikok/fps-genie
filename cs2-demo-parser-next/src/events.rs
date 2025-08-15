//! Events module - Game event parsing and processing  
//!
//! This module handles parsing of game events from demo data.
//! Phase 2 implementation with comprehensive event processing and analysis.

use crate::common::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive CS2 game events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CS2GameEvent {
    // Round events
    RoundStart {
        time_limit: u32,
        frag_limit: u32,
        objective: String,
    },
    RoundEnd {
        winner: Team,
        reason: RoundEndReason,
        message: String,
        round_time: f32,
    },
    RoundFreezeEnd,
    RoundAnnounceMatchStart,
    
    // Player events
    PlayerConnect {
        user_id: u32,
        name: String,
        steam_id: u64,
        team: Team,
    },
    PlayerDisconnect {
        user_id: u32,
        reason: String,
        steam_id: u64,
    },
    PlayerTeam {
        user_id: u32,
        team: Team,
        old_team: Team,
        disconnect: bool,
    },
    PlayerSpawn {
        user_id: u32,
        team: Team,
    },
    PlayerDeath {
        user_id: u32,
        attacker: u32,
        assister: Option<u32>,
        weapon: String,
        weapon_itemid: Option<u32>,
        headshot: bool,
        dominated: bool,
        revenge: bool,
        penetrated: u32,
        noreplay: bool,
        noscope: bool,
        thrusmoke: bool,
        attackerblind: bool,
        distance: f32,
    },
    PlayerHurt {
        user_id: u32,
        attacker: u32,
        health: u32,
        armor: u32,
        weapon: String,
        dmg_health: u32,
        dmg_armor: u32,
        hitgroup: HitGroup,
    },
    PlayerBlind {
        user_id: u32,
        attacker: u32,
        blind_duration: f32,
    },
    PlayerFootstep {
        user_id: u32,
        position: [f32; 3],
    },
    PlayerJump {
        user_id: u32,
    },
    
    // Weapon events
    WeaponFire {
        user_id: u32,
        weapon: String,
        silenced: bool,
    },
    WeaponReload {
        user_id: u32,
        weapon: String,
    },
    WeaponZoom {
        user_id: u32,
        weapon: String,
        zoom_level: u32,
    },
    
    // Grenade events
    HegrenadeDetonate {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    FlashbangDetonate {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    SmokegrenadeDetonate {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    SmokegrenadeExpired {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    DecoyDetonate {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    MolotovDetonate {
        user_id: u32,
        entity_id: u32,
        position: [f32; 3],
    },
    InfernoStartburn {
        entity_id: u32,
        position: [f32; 3],
    },
    InfernoExpire {
        entity_id: u32,
    },
    
    // Bomb events
    BombBeginplant {
        user_id: u32,
        site: BombSite,
    },
    BombAbortplant {
        user_id: u32,
        site: BombSite,
    },
    BombPlanted {
        user_id: u32,
        site: BombSite,
    },
    BombBegindefuse {
        user_id: u32,
        has_kit: bool,
    },
    BombAbortdefuse {
        user_id: u32,
        has_kit: bool,
    },
    BombDefused {
        user_id: u32,
        has_kit: bool,
        site: BombSite,
    },
    BombExploded {
        user_id: u32,
        site: BombSite,
    },
    BombDropped {
        user_id: u32,
        entity_id: u32,
    },
    BombPickup {
        user_id: u32,
    },
    
    // Buy events
    ItemPurchase {
        user_id: u32,
        team: Team,
        weapon: String,
    },
    ItemPickup {
        user_id: u32,
        item: String,
        silent: bool,
    },
    ItemDrop {
        user_id: u32,
        item: String,
    },
    
    // Hostage events (for Hostage maps)
    HostageFollows {
        user_id: u32,
        hostage: u32,
    },
    HostageHurt {
        user_id: u32,
        hostage: u32,
    },
    HostageKilled {
        user_id: u32,
        hostage: u32,
    },
    HostageRescued {
        user_id: u32,
        hostage: u32,
        site: u32,
    },
    
    // Other events
    PlayerMoneyChanged {
        user_id: u32,
        money: u32,
    },
    PlayerRadio {
        user_id: u32,
        slot: u32,
        subslot: u32,
        pos_x: f32,
        pos_y: f32,
        pos_z: f32,
    },
    
    // Custom event for unknown events
    Unknown {
        event_name: String,
        data: HashMap<String, String>,
    },
}

/// Team enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Team {
    Unassigned = 0,
    Spectator = 1,
    Terrorist = 2,
    CounterTerrorist = 3,
}

impl From<u32> for Team {
    fn from(value: u32) -> Self {
        match value {
            0 => Team::Unassigned,
            1 => Team::Spectator,
            2 => Team::Terrorist,
            3 => Team::CounterTerrorist,
            _ => Team::Unassigned,
        }
    }
}

/// Round end reasons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoundEndReason {
    TargetBombed = 1,
    VipEscaped = 2,
    VipAssassinated = 3,
    TerroristsEscaped = 4,
    CtStoppedEscape = 5,
    TerroristsStopped = 6,
    BombDefused = 7,
    CtWin = 8,
    TerroristWin = 9,
    Draw = 10,
    HostagesRescued = 11,
    TargetSaved = 12,
    HostagesNotRescued = 13,
    TerroristsNotEscaped = 14,
    VipNotEscaped = 15,
    GameStart = 16,
    TerrorristTimeOut = 17,
    CtTimeOut = 18,
    SurvivalDraw = 19,
    SurvivalWin = 20,
}

/// Hit groups for damage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HitGroup {
    Generic = 0,
    Head = 1,
    Chest = 2,
    Stomach = 3,
    LeftArm = 4,
    RightArm = 5,
    LeftLeg = 6,
    RightLeg = 7,
    Gear = 10,
}

/// Bomb sites
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BombSite {
    A = 0,
    B = 1,
}

/// Event timing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTiming {
    pub tick: u32,
    pub server_time: f32,
    pub round_time: f32,
}

/// Event metadata for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: u32,
    pub priority: EventPriority,
    pub impact_score: f32,
    pub involved_players: Vec<u32>,
    pub location: Option<[f32; 3]>,
}

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EventPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Complete event record with timing and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub event: CS2GameEvent,
    pub timing: EventTiming,
    pub metadata: EventMetadata,
}

/// Advanced event manager for processing and analyzing game events
pub struct EventManager {
    /// All events with their timing information
    events: Vec<EventRecord>,
    
    /// Event lookup by tick for fast access
    events_by_tick: HashMap<u32, Vec<usize>>,
    
    /// Player event tracking
    player_events: HashMap<u32, Vec<usize>>, // user_id -> event indices
    
    /// Round event tracking
    round_events: Vec<Vec<usize>>, // round_number -> event indices
    
    /// Current round number
    current_round: u32,
    
    /// Event statistics
    event_counts: HashMap<String, u32>,
    
    /// Performance tracking
    total_events: u64,
    processing_time_ms: u64,
}

impl EventManager {
    /// Create new event manager
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
            events_by_tick: HashMap::new(),
            player_events: HashMap::new(),
            round_events: Vec::new(),
            current_round: 0,
            event_counts: HashMap::new(),
            total_events: 0,
            processing_time_ms: 0,
        }
    }
    
    /// Add event at specific tick
    pub fn add_event(&mut self, tick: u32, event: CS2GameEvent) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Calculate metadata
        let metadata = self.calculate_event_metadata(&event, tick);
        let timing = EventTiming {
            tick,
            server_time: tick as f32 / 64.0, // Assuming 64 tick server
            round_time: 0.0, // Would be calculated from round start
        };
        
        let event_record = EventRecord {
            event: event.clone(),
            timing,
            metadata,
        };
        
        // Store event
        let event_index = self.events.len();
        self.events.push(event_record);
        
        // Update indices
        self.events_by_tick
            .entry(tick)
            .or_insert_with(Vec::new)
            .push(event_index);
        
        // Update player tracking
        let involved_players = self.get_involved_players(&event);
        for player_id in involved_players {
            self.player_events
                .entry(player_id)
                .or_insert_with(Vec::new)
                .push(event_index);
        }
        
        // Update round tracking
        if let CS2GameEvent::RoundStart { .. } = event {
            self.current_round += 1;
            self.round_events.push(Vec::new());
        }
        
        if self.current_round > 0 && (self.current_round as usize) <= self.round_events.len() {
            let round_index = (self.current_round - 1) as usize;
            if round_index < self.round_events.len() {
                self.round_events[round_index].push(event_index);
            }
        }
        
        // Update statistics
        let event_name = self.get_event_name(&event);
        *self.event_counts.entry(event_name).or_insert(0) += 1;
        self.total_events += 1;
        
        self.processing_time_ms += start_time.elapsed().as_millis() as u64;
        Ok(())
    }
    
    /// Calculate event metadata
    fn calculate_event_metadata(&self, event: &CS2GameEvent, tick: u32) -> EventMetadata {
        let (priority, impact_score) = match event {
            CS2GameEvent::PlayerDeath { headshot, weapon, .. } => {
                let base_score = if *headshot { 1.5 } else { 1.0 };
                let weapon_multiplier = match weapon.as_str() {
                    "awp" => 1.5,
                    "ak47" | "m4a1" => 1.2,
                    "knife" => 2.0,
                    _ => 1.0,
                };
                (EventPriority::High, base_score * weapon_multiplier)
            }
            CS2GameEvent::BombPlanted { .. } | CS2GameEvent::BombDefused { .. } => {
                (EventPriority::Critical, 3.0)
            }
            CS2GameEvent::BombExploded { .. } => {
                (EventPriority::Critical, 4.0)
            }
            CS2GameEvent::RoundEnd { .. } => {
                (EventPriority::Critical, 2.0)
            }
            CS2GameEvent::WeaponFire { .. } => {
                (EventPriority::Low, 0.1)
            }
            CS2GameEvent::PlayerHurt { dmg_health, .. } => {
                let impact = (*dmg_health as f32) / 100.0;
                (EventPriority::Medium, impact)
            }
            _ => (EventPriority::Medium, 0.5),
        };
        
        EventMetadata {
            event_id: tick, // Using tick as event ID for now
            priority,
            impact_score,
            involved_players: self.get_involved_players(event),
            location: self.get_event_location(event),
        }
    }
    
    /// Get players involved in an event
    fn get_involved_players(&self, event: &CS2GameEvent) -> Vec<u32> {
        match event {
            CS2GameEvent::PlayerDeath { user_id, attacker, assister, .. } => {
                let mut players = vec![*user_id, *attacker];
                if let Some(assist) = assister {
                    players.push(*assist);
                }
                players
            }
            CS2GameEvent::PlayerHurt { user_id, attacker, .. } => {
                vec![*user_id, *attacker]
            }
            CS2GameEvent::PlayerConnect { user_id, .. } |
            CS2GameEvent::PlayerDisconnect { user_id, .. } |
            CS2GameEvent::PlayerSpawn { user_id, .. } |
            CS2GameEvent::WeaponFire { user_id, .. } |
            CS2GameEvent::BombPlanted { user_id, .. } |
            CS2GameEvent::BombDefused { user_id, .. } => {
                vec![*user_id]
            }
            _ => Vec::new(),
        }
    }
    
    /// Get event location if available
    fn get_event_location(&self, event: &CS2GameEvent) -> Option<[f32; 3]> {
        match event {
            CS2GameEvent::HegrenadeDetonate { position, .. } |
            CS2GameEvent::FlashbangDetonate { position, .. } |
            CS2GameEvent::SmokegrenadeDetonate { position, .. } |
            CS2GameEvent::PlayerFootstep { position, .. } => Some(*position),
            _ => None,
        }
    }
    
    /// Get event name for statistics
    fn get_event_name(&self, event: &CS2GameEvent) -> String {
        match event {
            CS2GameEvent::RoundStart { .. } => "round_start".to_string(),
            CS2GameEvent::RoundEnd { .. } => "round_end".to_string(),
            CS2GameEvent::PlayerConnect { .. } => "player_connect".to_string(),
            CS2GameEvent::PlayerDisconnect { .. } => "player_disconnect".to_string(),
            CS2GameEvent::PlayerDeath { .. } => "player_death".to_string(),
            CS2GameEvent::PlayerHurt { .. } => "player_hurt".to_string(),
            CS2GameEvent::WeaponFire { .. } => "weapon_fire".to_string(),
            CS2GameEvent::BombPlanted { .. } => "bomb_planted".to_string(),
            CS2GameEvent::BombDefused { .. } => "bomb_defused".to_string(),
            CS2GameEvent::BombExploded { .. } => "bomb_exploded".to_string(),
            CS2GameEvent::Unknown { event_name, .. } => event_name.clone(),
            _ => "other".to_string(),
        }
    }
    
    /// Get events for tick range
    pub fn get_events_in_range(&self, start_tick: u32, end_tick: u32) -> Vec<&EventRecord> {
        let mut events = Vec::new();
        for tick in start_tick..=end_tick {
            if let Some(indices) = self.events_by_tick.get(&tick) {
                for &index in indices {
                    if let Some(event) = self.events.get(index) {
                        events.push(event);
                    }
                }
            }
        }
        events
    }
    
    /// Get events at specific tick
    pub fn get_events_at_tick(&self, tick: u32) -> Vec<&EventRecord> {
        self.events_by_tick
            .get(&tick)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&index| self.events.get(index))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get events for specific player
    pub fn get_player_events(&self, user_id: u32) -> Vec<&EventRecord> {
        self.player_events
            .get(&user_id)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&index| self.events.get(index))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Get events for specific round
    pub fn get_round_events(&self, round_number: u32) -> Vec<&EventRecord> {
        if round_number == 0 || round_number as usize > self.round_events.len() {
            return Vec::new();
        }
        
        let round_index = (round_number - 1) as usize;
        self.round_events[round_index]
            .iter()
            .filter_map(|&index| self.events.get(index))
            .collect()
    }
    
    /// Get events by priority
    pub fn get_events_by_priority(&self, priority: EventPriority) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|event| event.metadata.priority == priority)
            .collect()
    }
    
    /// Get critical events (high impact)
    pub fn get_critical_events(&self) -> Vec<&EventRecord> {
        self.events
            .iter()
            .filter(|event| event.metadata.impact_score >= 2.0)
            .collect()
    }
    
    /// Get all events
    pub fn events(&self) -> &Vec<EventRecord> {
        &self.events
    }
    
    /// Get event statistics
    pub fn get_event_statistics(&self) -> HashMap<String, u32> {
        self.event_counts.clone()
    }
    
    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> (u64, u64, f64) {
        let avg_processing_time = if self.total_events > 0 {
            self.processing_time_ms as f64 / self.total_events as f64
        } else {
            0.0
        };
        (self.total_events, self.processing_time_ms, avg_processing_time)
    }
    
    /// Find kill streaks for a player
    pub fn find_kill_streaks(&self, user_id: u32, min_kills: u32) -> Vec<Vec<&EventRecord>> {
        let player_events = self.get_player_events(user_id);
        let mut kill_streaks = Vec::new();
        let mut current_streak = Vec::new();
        
        for event in player_events {
            match &event.event {
                CS2GameEvent::PlayerDeath { attacker, .. } if *attacker == user_id => {
                    current_streak.push(event);
                }
                CS2GameEvent::PlayerDeath { user_id: victim, .. } if *victim == user_id => {
                    // Player died, end streak
                    if current_streak.len() >= min_kills as usize {
                        kill_streaks.push(current_streak.clone());
                    }
                    current_streak.clear();
                }
                CS2GameEvent::RoundEnd { .. } => {
                    // Round ended, check streak
                    if current_streak.len() >= min_kills as usize {
                        kill_streaks.push(current_streak.clone());
                    }
                    current_streak.clear();
                }
                _ => {}
            }
        }
        
        // Check final streak
        if current_streak.len() >= min_kills as usize {
            kill_streaks.push(current_streak);
        }
        
        kill_streaks
    }
    
    /// Analyze round performance
    pub fn analyze_round_performance(&self, round_number: u32) -> RoundAnalysis {
        let round_events = self.get_round_events(round_number);
        
        let mut analysis = RoundAnalysis {
            round_number,
            total_events: round_events.len(),
            kills: 0,
            deaths: 0,
            bomb_plants: 0,
            bomb_defuses: 0,
            grenade_throws: 0,
            weapon_fires: 0,
            damage_dealt: 0,
            round_winner: Team::Unassigned,
            round_end_reason: RoundEndReason::Draw,
            round_duration: 0.0,
        };
        
        for event in round_events {
            match &event.event {
                CS2GameEvent::PlayerDeath { .. } => {
                    analysis.kills += 1;
                    analysis.deaths += 1;
                }
                CS2GameEvent::BombPlanted { .. } => {
                    analysis.bomb_plants += 1;
                }
                CS2GameEvent::BombDefused { .. } => {
                    analysis.bomb_defuses += 1;
                }
                CS2GameEvent::WeaponFire { .. } => {
                    analysis.weapon_fires += 1;
                }
                CS2GameEvent::PlayerHurt { dmg_health, .. } => {
                    analysis.damage_dealt += *dmg_health;
                }
                CS2GameEvent::RoundEnd { winner, reason, .. } => {
                    analysis.round_winner = *winner;
                    analysis.round_end_reason = *reason;
                }
                CS2GameEvent::HegrenadeDetonate { .. } |
                CS2GameEvent::FlashbangDetonate { .. } |
                CS2GameEvent::SmokegrenadeDetonate { .. } => {
                    analysis.grenade_throws += 1;
                }
                _ => {}
            }
        }
        
        analysis
    }
}

/// Round analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundAnalysis {
    pub round_number: u32,
    pub total_events: usize,
    pub kills: u32,
    pub deaths: u32,
    pub bomb_plants: u32,
    pub bomb_defuses: u32,
    pub grenade_throws: u32,
    pub weapon_fires: u32,
    pub damage_dealt: u32,
    pub round_winner: Team,
    pub round_end_reason: RoundEndReason,
    pub round_duration: f32,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_event_manager_creation() {
        let manager = EventManager::new();
        assert_eq!(manager.events().len(), 0);
        assert_eq!(manager.current_round, 0);
    }
    
    #[test]
    fn test_add_event() {
        let mut manager = EventManager::new();
        
        let event = CS2GameEvent::PlayerConnect {
            user_id: 1,
            name: "test_player".to_string(),
            steam_id: 76561198000000000,
            team: Team::Terrorist,
        };
        
        manager.add_event(100, event).unwrap();
        assert_eq!(manager.events().len(), 1);
        
        let events_at_tick = manager.get_events_at_tick(100);
        assert_eq!(events_at_tick.len(), 1);
    }
    
    #[test]
    fn test_round_tracking() {
        let mut manager = EventManager::new();
        
        // Start round 1
        manager.add_event(1000, CS2GameEvent::RoundStart {
            time_limit: 115,
            frag_limit: 0,
            objective: "Bomb defusal".to_string(),
        }).unwrap();
        
        // Add some events
        manager.add_event(1100, CS2GameEvent::PlayerDeath {
            user_id: 1,
            attacker: 2,
            assister: None,
            weapon: "ak47".to_string(),
            weapon_itemid: None,
            headshot: true,
            dominated: false,
            revenge: false,
            penetrated: 0,
            noreplay: false,
            noscope: false,
            thrusmoke: false,
            attackerblind: false,
            distance: 25.5,
        }).unwrap();
        
        // End round
        manager.add_event(2000, CS2GameEvent::RoundEnd {
            winner: Team::Terrorist,
            reason: RoundEndReason::TerroristWin,
            message: "Terrorists Win".to_string(),
            round_time: 45.0,
        }).unwrap();
        
        assert_eq!(manager.current_round, 1);
        let round_events = manager.get_round_events(1);
        assert_eq!(round_events.len(), 3);
    }
    
    #[test]
    fn test_player_event_tracking() {
        let mut manager = EventManager::new();
        
        let event1 = CS2GameEvent::PlayerConnect {
            user_id: 1,
            name: "player1".to_string(),
            steam_id: 76561198000000001,
            team: Team::Terrorist,
        };
        
        let event2 = CS2GameEvent::PlayerDeath {
            user_id: 1,
            attacker: 2,
            assister: None,
            weapon: "ak47".to_string(),
            weapon_itemid: None,
            headshot: false,
            dominated: false,
            revenge: false,
            penetrated: 0,
            noreplay: false,
            noscope: false,
            thrusmoke: false,
            attackerblind: false,
            distance: 15.0,
        };
        
        manager.add_event(100, event1).unwrap();
        manager.add_event(200, event2).unwrap();
        
        let player_events = manager.get_player_events(1);
        assert_eq!(player_events.len(), 2);
        
        let attacker_events = manager.get_player_events(2);
        assert_eq!(attacker_events.len(), 1);
    }
    
    #[test]
    fn test_event_priority_calculation() {
        let mut manager = EventManager::new();
        
        let critical_event = CS2GameEvent::BombPlanted {
            user_id: 1,
            site: BombSite::A,
        };
        
        let low_event = CS2GameEvent::WeaponFire {
            user_id: 1,
            weapon: "ak47".to_string(),
            silenced: false,
        };
        
        manager.add_event(100, critical_event).unwrap();
        manager.add_event(101, low_event).unwrap();
        
        let critical_events = manager.get_events_by_priority(EventPriority::Critical);
        assert_eq!(critical_events.len(), 1);
        
        let low_events = manager.get_events_by_priority(EventPriority::Low);
        assert_eq!(low_events.len(), 1);
    }
    
    #[test]
    fn test_event_statistics() {
        let mut manager = EventManager::new();
        
        manager.add_event(100, CS2GameEvent::PlayerDeath {
            user_id: 1,
            attacker: 2,
            assister: None,
            weapon: "ak47".to_string(),
            weapon_itemid: None,
            headshot: true,
            dominated: false,
            revenge: false,
            penetrated: 0,
            noreplay: false,
            noscope: false,
            thrusmoke: false,
            attackerblind: false,
            distance: 20.0,
        }).unwrap();
        
        manager.add_event(101, CS2GameEvent::PlayerDeath {
            user_id: 3,
            attacker: 4,
            assister: None,
            weapon: "awp".to_string(),
            weapon_itemid: None,
            headshot: true,
            dominated: false,
            revenge: false,
            penetrated: 0,
            noreplay: false,
            noscope: false,
            thrusmoke: false,
            attackerblind: false,
            distance: 50.0,
        }).unwrap();
        
        let stats = manager.get_event_statistics();
        assert_eq!(stats.get("player_death"), Some(&2));
    }
    
    #[test]
    fn test_team_conversion() {
        assert_eq!(Team::from(2), Team::Terrorist);
        assert_eq!(Team::from(3), Team::CounterTerrorist);
        assert_eq!(Team::from(999), Team::Unassigned);
    }
}