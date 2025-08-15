//! Game state module - Tracking overall game state
//!
//! This module manages the overall game state derived from demo parsing.
//! Phase 3 implementation with comprehensive state tracking and coaching insights.

use crate::common::{Error, Result};
use crate::entities::{EntityManager};
use crate::events::{EventManager, CS2GameEvent, Team, RoundEndReason};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive game state tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current tick
    pub current_tick: u32,
    
    /// Current round number
    pub round_number: u32,
    
    /// Game phase (warmup, live, etc.)
    pub game_phase: GamePhase,
    
    /// Score information
    pub score: Score,
    
    /// Round state
    pub round_state: RoundState,
    
    /// Player states
    pub players: HashMap<u32, PlayerState>,
    
    /// Team states
    pub teams: HashMap<Team, TeamState>,
    
    /// Economy state
    pub economy: EconomyState,
    
    /// Map information
    pub map_info: MapInfo,
    
    /// Bomb state (for bomb defusal maps)
    pub bomb_state: BombState,
    
    /// Server information
    pub server_info: ServerInfo,
}

/// Game phase enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    WarmupPeriod,
    StartGamePeriod,
    TeamIntro,
    KnifeRound,
    GameHalftime,
    GameCommencing,
    Live,
    Halftime,
    Overtime,
    PostGame,
    Finished,
}

/// Detailed score tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    /// Team scores
    pub team_scores: HashMap<Team, u32>,
    
    /// Round history
    pub round_history: Vec<RoundResult>,
    
    /// Match format info
    pub match_format: MatchFormat,
    
    /// Overtime information
    pub overtime_periods: u32,
}

/// Round result information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundResult {
    pub round_number: u32,
    pub winner: Team,
    pub reason: RoundEndReason,
    pub duration: f32,
    pub end_tick: u32,
}

/// Match format information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchFormat {
    pub max_rounds: u32,
    pub rounds_to_win: u32,
    pub overtime_max_rounds: u32,
    pub overtime_rounds_to_win: u32,
}

/// Round state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundState {
    /// Current round phase
    pub phase: RoundPhase,
    
    /// Round start tick
    pub start_tick: u32,
    
    /// Round time remaining (seconds)
    pub time_remaining: f32,
    
    /// Round time limit
    pub time_limit: f32,
    
    /// Buy time remaining
    pub buy_time_remaining: f32,
    
    /// Freeze time remaining
    pub freeze_time_remaining: f32,
    
    /// Living players count per team
    pub living_players: HashMap<Team, u32>,
    
    /// Bomb planted status
    pub bomb_planted: bool,
    
    /// Bomb planted tick
    pub bomb_planted_tick: Option<u32>,
    
    /// Bomb site (if planted)
    pub bomb_site: Option<String>,
}

/// Round phase enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoundPhase {
    Freeze,
    Buy,
    Live,
    PostRound,
}

/// Player state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    /// Player basic info
    pub user_id: u32,
    pub steam_id: u64,
    pub name: String,
    pub team: Team,
    
    /// Player entity reference
    pub entity_id: Option<u32>,
    
    /// Health and armor
    pub health: u32,
    pub armor: u32,
    pub has_helmet: bool,
    
    /// Position and movement
    pub position: Option<[f32; 3]>,
    pub velocity: Option<[f32; 3]>,
    pub view_angles: Option<[f32; 3]>,
    
    /// Weapon information
    pub active_weapon: Option<String>,
    pub weapons: Vec<WeaponInfo>,
    pub grenades: Vec<GrenadeInfo>,
    
    /// Economy
    pub money: u32,
    pub equipment_value: u32,
    
    /// Statistics this round
    pub round_stats: PlayerRoundStats,
    
    /// Overall match statistics
    pub match_stats: PlayerMatchStats,
    
    /// Status flags
    pub is_alive: bool,
    pub is_connected: bool,
    pub is_bot: bool,
    pub is_spectating: bool,
    pub is_defusing: bool,
    pub is_planting: bool,
    pub is_scoped: bool,
    pub is_walking: bool,
    pub is_ducking: bool,
    
    /// Coaching insights
    pub performance_metrics: PlayerPerformanceMetrics,
}

/// Weapon information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponInfo {
    pub name: String,
    pub ammo_clip: u32,
    pub ammo_reserve: u32,
    pub skin_id: Option<u32>,
}

/// Grenade information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrenadeInfo {
    pub name: String,
    pub count: u32,
}

/// Player round statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerRoundStats {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
    pub damage_dealt: u32,
    pub damage_received: u32,
    pub shots_fired: u32,
    pub shots_hit: u32,
    pub headshots: u32,
    pub grenades_thrown: u32,
    pub money_spent: u32,
}

/// Player match statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMatchStats {
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_assists: u32,
    pub total_damage: u32,
    pub total_headshots: u32,
    pub adr: f32, // Average damage per round
    pub kdr: f32, // Kill/death ratio
    pub hsr: f32, // Headshot ratio
    pub rating: f32, // Overall performance rating
}

/// Player performance metrics for coaching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerPerformanceMetrics {
    /// Aim accuracy metrics
    pub accuracy: f32,
    pub headshot_percentage: f32,
    
    /// Positioning metrics
    pub crosshair_placement_score: f32,
    pub positioning_score: f32,
    
    /// Decision making metrics
    pub decision_making_score: f32,
    pub utility_usage_score: f32,
    
    /// Economic metrics
    pub economy_management_score: f32,
    
    /// Team play metrics
    pub teamwork_score: f32,
    
    /// Overall coaching rating
    pub coaching_rating: f32,
    
    /// Areas for improvement
    pub improvement_areas: Vec<String>,
}

/// Team state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamState {
    pub team: Team,
    pub score: u32,
    pub living_players: u32,
    pub total_money: u32,
    pub average_health: f32,
    pub equipment_value: u32,
    pub utility_count: u32,
    pub team_tactics: TeamTactics,
}

/// Team tactics analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamTactics {
    pub formation: String,
    pub aggression_level: f32,
    pub map_control_percentage: f32,
    pub utility_coordination: f32,
    pub economic_strategy: String,
}

/// Economy state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomyState {
    pub round_type: EconomyRoundType,
    pub team_economies: HashMap<Team, TeamEconomy>,
    pub force_buy_threshold: u32,
    pub eco_round_threshold: u32,
}

/// Economy round types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EconomyRoundType {
    Pistol,
    Eco,
    ForceBuy,
    FullBuy,
    Save,
}

/// Team economy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamEconomy {
    pub total_money: u32,
    pub average_money: f32,
    pub equipment_value: u32,
    pub loss_bonus: u32,
    pub economy_rating: f32,
}

/// Map information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapInfo {
    pub name: String,
    pub mode: String,
    pub bomb_sites: Vec<String>,
    pub spawn_positions: HashMap<Team, Vec<[f32; 3]>>,
    pub callouts: HashMap<String, [f32; 3]>,
}

/// Bomb state for defusal maps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BombState {
    pub is_planted: bool,
    pub plant_tick: Option<u32>,
    pub site: Option<String>,
    pub position: Option<[f32; 3]>,
    pub time_remaining: Option<f32>,
    pub defuse_progress: f32,
    pub carrier_id: Option<u32>,
}

/// Server information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub ip: String,
    pub tickrate: u32,
    pub game_mode: String,
    pub max_players: u32,
}

/// Advanced game state manager with coaching capabilities
pub struct GameStateManager {
    /// Current game state
    state: GameState,
    
    /// Entity manager reference
    entity_manager: EntityManager,
    
    /// Event manager reference
    event_manager: EventManager,
    
    /// State history for analysis
    state_history: Vec<(u32, GameState)>, // (tick, state)
    
    /// Performance tracking
    update_count: u64,
    processing_time_ms: u64,
    
    /// Coaching analysis cache
    coaching_cache: HashMap<u32, CoachingInsights>, // user_id -> insights
}

/// Coaching insights for players
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachingInsights {
    pub player_id: u32,
    pub generated_tick: u32,
    
    /// Performance analysis
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub recommendations: Vec<String>,
    
    /// Comparative analysis
    pub rank_comparison: RankComparison,
    pub pro_comparison: ProComparison,
    
    /// Specific improvement areas
    pub aim_training_suggestions: Vec<String>,
    pub positioning_tips: Vec<String>,
    pub utility_usage_tips: Vec<String>,
    pub economic_advice: Vec<String>,
}

/// Rank-based comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankComparison {
    pub estimated_rank: String,
    pub percentile: f32,
    pub rank_progression_tips: Vec<String>,
}

/// Professional player comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProComparison {
    pub similar_pro_players: Vec<String>,
    pub playstyle_analysis: String,
    pub professional_techniques: Vec<String>,
}

impl GameStateManager {
    /// Create new game state manager
    pub fn new() -> Self {
        Self {
            state: GameState {
                current_tick: 0,
                round_number: 0,
                game_phase: GamePhase::WarmupPeriod,
                score: Score {
                    team_scores: HashMap::new(),
                    round_history: Vec::new(),
                    match_format: MatchFormat {
                        max_rounds: 30,
                        rounds_to_win: 16,
                        overtime_max_rounds: 6,
                        overtime_rounds_to_win: 4,
                    },
                    overtime_periods: 0,
                },
                round_state: RoundState {
                    phase: RoundPhase::Freeze,
                    start_tick: 0,
                    time_remaining: 115.0,
                    time_limit: 115.0,
                    buy_time_remaining: 15.0,
                    freeze_time_remaining: 5.0,
                    living_players: HashMap::new(),
                    bomb_planted: false,
                    bomb_planted_tick: None,
                    bomb_site: None,
                },
                players: HashMap::new(),
                teams: HashMap::new(),
                economy: EconomyState {
                    round_type: EconomyRoundType::Pistol,
                    team_economies: HashMap::new(),
                    force_buy_threshold: 3000,
                    eco_round_threshold: 1500,
                },
                map_info: MapInfo {
                    name: "de_dust2".to_string(),
                    mode: "defusal".to_string(),
                    bomb_sites: vec!["A".to_string(), "B".to_string()],
                    spawn_positions: HashMap::new(),
                    callouts: HashMap::new(),
                },
                bomb_state: BombState {
                    is_planted: false,
                    plant_tick: None,
                    site: None,
                    position: None,
                    time_remaining: None,
                    defuse_progress: 0.0,
                    carrier_id: None,
                },
                server_info: ServerInfo {
                    name: "Demo Server".to_string(),
                    ip: "127.0.0.1".to_string(),
                    tickrate: 64,
                    game_mode: "competitive".to_string(),
                    max_players: 10,
                },
            },
            entity_manager: EntityManager::new(),
            event_manager: EventManager::new(),
            state_history: Vec::new(),
            update_count: 0,
            processing_time_ms: 0,
            coaching_cache: HashMap::new(),
        }
    }
    
    /// Update state from current tick
    pub fn update_state(&mut self, tick: u32) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        self.state.current_tick = tick;
        
        // Update player states from entities
        self.update_player_states()?;
        
        // Update round state
        self.update_round_state()?;
        
        // Update team states
        self.update_team_states()?;
        
        // Update economy
        self.update_economy_state()?;
        
        // Update bomb state
        self.update_bomb_state()?;
        
        // Store state history (sample every 64 ticks for performance)
        if tick % 64 == 0 {
            self.state_history.push((tick, self.state.clone()));
            
            // Limit history size for memory management
            if self.state_history.len() > 1000 {
                self.state_history.remove(0);
            }
        }
        
        self.update_count += 1;
        self.processing_time_ms += start_time.elapsed().as_millis() as u64;
        
        Ok(())
    }
    
    /// Update player states from entity data
    fn update_player_states(&mut self) -> Result<()> {
        for entity in self.entity_manager.get_player_entities() {
            if let Some(user_id) = entity.properties.get("m_iUserID") {
                if let crate::entities::EntityProperty::Int(uid) = user_id {
                    let user_id = *uid as u32;
                    
                    let player_state = self.state.players.entry(user_id).or_insert_with(|| {
                        PlayerState {
                            user_id,
                            steam_id: 0,
                            name: "Unknown".to_string(),
                            team: Team::Unassigned,
                            entity_id: Some(entity.id),
                            health: 100,
                            armor: 0,
                            has_helmet: false,
                            position: None,
                            velocity: None,
                            view_angles: None,
                            active_weapon: None,
                            weapons: Vec::new(),
                            grenades: Vec::new(),
                            money: 800,
                            equipment_value: 0,
                            round_stats: PlayerRoundStats {
                                kills: 0,
                                deaths: 0,
                                assists: 0,
                                damage_dealt: 0,
                                damage_received: 0,
                                shots_fired: 0,
                                shots_hit: 0,
                                headshots: 0,
                                grenades_thrown: 0,
                                money_spent: 0,
                            },
                            match_stats: PlayerMatchStats {
                                total_kills: 0,
                                total_deaths: 0,
                                total_assists: 0,
                                total_damage: 0,
                                total_headshots: 0,
                                adr: 0.0,
                                kdr: 0.0,
                                hsr: 0.0,
                                rating: 1.0,
                            },
                            is_alive: true,
                            is_connected: true,
                            is_bot: false,
                            is_spectating: false,
                            is_defusing: false,
                            is_planting: false,
                            is_scoped: false,
                            is_walking: false,
                            is_ducking: false,
                            performance_metrics: PlayerPerformanceMetrics {
                                accuracy: 0.0,
                                headshot_percentage: 0.0,
                                crosshair_placement_score: 0.0,
                                positioning_score: 0.0,
                                decision_making_score: 0.0,
                                utility_usage_score: 0.0,
                                economy_management_score: 0.0,
                                teamwork_score: 0.0,
                                coaching_rating: 0.0,
                                improvement_areas: Vec::new(),
                            },
                        }
                    });
                    
                    // Update from entity properties
                    if let Some(health) = entity.health {
                        player_state.health = health;
                        player_state.is_alive = health > 0;
                    }
                    
                    if let Some(armor) = entity.armor {
                        player_state.armor = armor;
                    }
                    
                    if let Some(team) = entity.team {
                        player_state.team = Team::from(team);
                    }
                    
                    player_state.position = entity.position;
                    player_state.velocity = entity.velocity;
                }
            }
        }
        
        Ok(())
    }
    
    /// Update round state information
    fn update_round_state(&mut self) -> Result<()> {
        // Update living players count
        self.state.round_state.living_players.clear();
        for player in self.state.players.values() {
            if player.is_alive && player.team != Team::Unassigned {
                *self.state.round_state.living_players.entry(player.team).or_insert(0) += 1;
            }
        }
        
        // Update round time (simplified calculation)
        if self.state.round_state.start_tick > 0 {
            let elapsed_ticks = self.state.current_tick - self.state.round_state.start_tick;
            let elapsed_seconds = elapsed_ticks as f32 / 64.0; // Assuming 64 tick server
            self.state.round_state.time_remaining = 
                (self.state.round_state.time_limit - elapsed_seconds).max(0.0);
        }
        
        Ok(())
    }
    
    /// Update team states
    fn update_team_states(&mut self) -> Result<()> {
        for team in [Team::Terrorist, Team::CounterTerrorist] {
            let team_players: Vec<_> = self.state.players.values()
                .filter(|p| p.team == team)
                .collect();
            
            if !team_players.is_empty() {
                let living_count = team_players.iter().filter(|p| p.is_alive).count() as u32;
                let total_money = team_players.iter().map(|p| p.money).sum();
                let avg_health = team_players.iter()
                    .filter(|p| p.is_alive)
                    .map(|p| p.health as f32)
                    .sum::<f32>() / living_count.max(1) as f32;
                let equipment_value = team_players.iter().map(|p| p.equipment_value).sum();
                
                let team_state = TeamState {
                    team,
                    score: *self.state.score.team_scores.get(&team).unwrap_or(&0),
                    living_players: living_count,
                    total_money,
                    average_health: if living_count > 0 { avg_health } else { 0.0 },
                    equipment_value,
                    utility_count: 0, // Would be calculated from player inventories
                    team_tactics: TeamTactics {
                        formation: "Default".to_string(),
                        aggression_level: 0.5,
                        map_control_percentage: 50.0,
                        utility_coordination: 0.5,
                        economic_strategy: "Standard".to_string(),
                    },
                };
                
                self.state.teams.insert(team, team_state);
            }
        }
        
        Ok(())
    }
    
    /// Update economy state
    fn update_economy_state(&mut self) -> Result<()> {
        for team in [Team::Terrorist, Team::CounterTerrorist] {
            let team_players: Vec<_> = self.state.players.values()
                .filter(|p| p.team == team)
                .collect();
            
            if !team_players.is_empty() {
                let total_money = team_players.iter().map(|p| p.money).sum();
                let avg_money = total_money as f32 / team_players.len() as f32;
                let equipment_value = team_players.iter().map(|p| p.equipment_value).sum();
                
                let team_economy = TeamEconomy {
                    total_money,
                    average_money: avg_money,
                    equipment_value,
                    loss_bonus: 0, // Would be calculated based on round history
                    economy_rating: avg_money / 5000.0, // Normalized rating
                };
                
                self.state.economy.team_economies.insert(team, team_economy);
            }
        }
        
        Ok(())
    }
    
    /// Update bomb state
    fn update_bomb_state(&mut self) -> Result<()> {
        // This would be updated based on bomb-related events
        // For now, maintain existing state
        Ok(())
    }
    
    /// Process game event and update state accordingly
    pub fn process_event(&mut self, tick: u32, event: CS2GameEvent) -> Result<()> {
        // Add event to event manager
        self.event_manager.add_event(tick, event.clone())?;
        
        // Update state based on event
        match event {
            CS2GameEvent::RoundStart { time_limit, .. } => {
                self.state.round_number += 1;
                self.state.round_state.start_tick = tick;
                self.state.round_state.time_limit = time_limit as f32;
                self.state.round_state.time_remaining = time_limit as f32;
                self.state.round_state.phase = RoundPhase::Freeze;
                
                // Reset round stats for all players
                for player in self.state.players.values_mut() {
                    player.round_stats = PlayerRoundStats {
                        kills: 0,
                        deaths: 0,
                        assists: 0,
                        damage_dealt: 0,
                        damage_received: 0,
                        shots_fired: 0,
                        shots_hit: 0,
                        headshots: 0,
                        grenades_thrown: 0,
                        money_spent: 0,
                    };
                }
            }
            
            CS2GameEvent::RoundEnd { winner, reason, round_time, .. } => {
                // Update team scores
                let current_score = self.state.score.team_scores.get(&winner).unwrap_or(&0);
                self.state.score.team_scores.insert(winner, current_score + 1);
                
                // Add to round history
                self.state.score.round_history.push(RoundResult {
                    round_number: self.state.round_number,
                    winner,
                    reason,
                    duration: round_time,
                    end_tick: tick,
                });
                
                self.state.round_state.phase = RoundPhase::PostRound;
            }
            
            CS2GameEvent::PlayerDeath { user_id, attacker, weapon, headshot, .. } => {
                // Update player stats
                if let Some(victim) = self.state.players.get_mut(&user_id) {
                    victim.round_stats.deaths += 1;
                    victim.match_stats.total_deaths += 1;
                    victim.is_alive = false;
                }
                
                if let Some(killer) = self.state.players.get_mut(&attacker) {
                    killer.round_stats.kills += 1;
                    killer.match_stats.total_kills += 1;
                    
                    if headshot {
                        killer.round_stats.headshots += 1;
                        killer.match_stats.total_headshots += 1;
                    }
                }
            }
            
            CS2GameEvent::PlayerHurt { user_id, attacker, dmg_health, .. } => {
                if let Some(victim) = self.state.players.get_mut(&user_id) {
                    victim.round_stats.damage_received += dmg_health;
                }
                
                if let Some(attacker_player) = self.state.players.get_mut(&attacker) {
                    attacker_player.round_stats.damage_dealt += dmg_health;
                    attacker_player.match_stats.total_damage += dmg_health;
                }
            }
            
            CS2GameEvent::BombPlanted { user_id, .. } => {
                self.state.bomb_state.is_planted = true;
                self.state.bomb_state.plant_tick = Some(tick);
                self.state.bomb_state.carrier_id = Some(user_id);
                self.state.round_state.bomb_planted = true;
                self.state.round_state.bomb_planted_tick = Some(tick);
            }
            
            CS2GameEvent::BombDefused { user_id, .. } => {
                self.state.bomb_state.is_planted = false;
                self.state.bomb_state.defuse_progress = 1.0;
            }
            
            _ => {
                // Handle other events as needed
            }
        }
        
        Ok(())
    }
    
    /// Generate coaching insights for a player
    pub fn generate_coaching_insights(&mut self, user_id: u32) -> Result<CoachingInsights> {
        if let Some(player) = self.state.players.get(&user_id) {
            let insights = CoachingInsights {
                player_id: user_id,
                generated_tick: self.state.current_tick,
                strengths: self.analyze_player_strengths(player),
                weaknesses: self.analyze_player_weaknesses(player),
                recommendations: self.generate_recommendations(player),
                rank_comparison: RankComparison {
                    estimated_rank: self.estimate_rank(player),
                    percentile: self.calculate_percentile(player),
                    rank_progression_tips: self.generate_rank_tips(player),
                },
                pro_comparison: ProComparison {
                    similar_pro_players: self.find_similar_pros(player),
                    playstyle_analysis: self.analyze_playstyle(player),
                    professional_techniques: self.suggest_pro_techniques(player),
                },
                aim_training_suggestions: self.suggest_aim_training(player),
                positioning_tips: self.suggest_positioning_tips(player),
                utility_usage_tips: self.suggest_utility_tips(player),
                economic_advice: self.suggest_economic_advice(player),
            };
            
            self.coaching_cache.insert(user_id, insights.clone());
            Ok(insights)
        } else {
            Err(Error::InvalidFormat(format!("Player {} not found", user_id)))
        }
    }
    
    /// Analyze player strengths
    fn analyze_player_strengths(&self, player: &PlayerState) -> Vec<String> {
        let mut strengths = Vec::new();
        
        if player.performance_metrics.accuracy > 0.7 {
            strengths.push("Excellent aim accuracy".to_string());
        }
        
        if player.performance_metrics.headshot_percentage > 0.5 {
            strengths.push("High headshot rate".to_string());
        }
        
        if player.performance_metrics.positioning_score > 0.8 {
            strengths.push("Good positioning sense".to_string());
        }
        
        if player.match_stats.kdr > 1.5 {
            strengths.push("Strong fragging ability".to_string());
        }
        
        if strengths.is_empty() {
            strengths.push("Consistent gameplay".to_string());
        }
        
        strengths
    }
    
    /// Analyze player weaknesses
    fn analyze_player_weaknesses(&self, player: &PlayerState) -> Vec<String> {
        let mut weaknesses = Vec::new();
        
        if player.performance_metrics.accuracy < 0.3 {
            weaknesses.push("Low aim accuracy - needs aim training".to_string());
        }
        
        if player.performance_metrics.positioning_score < 0.4 {
            weaknesses.push("Poor positioning - exposing to unnecessary risks".to_string());
        }
        
        if player.performance_metrics.economy_management_score < 0.5 {
            weaknesses.push("Inefficient economy management".to_string());
        }
        
        if player.match_stats.kdr < 0.8 {
            weaknesses.push("Below average fragging performance".to_string());
        }
        
        weaknesses
    }
    
    /// Generate improvement recommendations
    fn generate_recommendations(&self, player: &PlayerState) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if player.performance_metrics.accuracy < 0.5 {
            recommendations.push("Practice aim training maps like aim_botz".to_string());
            recommendations.push("Lower your sensitivity for better precision".to_string());
        }
        
        if player.performance_metrics.positioning_score < 0.6 {
            recommendations.push("Watch professional demos to learn proper positioning".to_string());
            recommendations.push("Practice crosshair placement at head level".to_string());
        }
        
        if player.performance_metrics.utility_usage_score < 0.5 {
            recommendations.push("Learn smoke and flash lineups for this map".to_string());
            recommendations.push("Coordinate utility usage with teammates".to_string());
        }
        
        recommendations.push("Review your own demos to identify mistakes".to_string());
        
        recommendations
    }
    
    /// Estimate player rank
    fn estimate_rank(&self, player: &PlayerState) -> String {
        let rating = player.match_stats.rating;
        
        match rating {
            r if r >= 1.4 => "Global Elite".to_string(),
            r if r >= 1.3 => "Supreme Master First Class".to_string(),
            r if r >= 1.2 => "Legendary Eagle Master".to_string(),
            r if r >= 1.1 => "Legendary Eagle".to_string(),
            r if r >= 1.0 => "Distinguished Master Guardian".to_string(),
            r if r >= 0.9 => "Master Guardian Elite".to_string(),
            r if r >= 0.8 => "Master Guardian".to_string(),
            r if r >= 0.7 => "Gold Nova Master".to_string(),
            r if r >= 0.6 => "Gold Nova III".to_string(),
            r if r >= 0.5 => "Gold Nova II".to_string(),
            r if r >= 0.4 => "Gold Nova I".to_string(),
            _ => "Silver Elite Master".to_string(),
        }
    }
    
    /// Calculate performance percentile
    fn calculate_percentile(&self, player: &PlayerState) -> f32 {
        // Simplified percentile calculation
        (player.match_stats.rating * 50.0).min(99.0)
    }
    
    /// Generate rank progression tips
    fn generate_rank_tips(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "Focus on consistency over flashy plays".to_string(),
            "Improve game sense through demo analysis".to_string(),
            "Practice aim daily for 30 minutes".to_string(),
            "Learn common angles and prefire spots".to_string(),
        ]
    }
    
    /// Find similar professional players
    fn find_similar_pros(&self, _player: &PlayerState) -> Vec<String> {
        // Placeholder implementation - would use machine learning in practice
        vec![
            "s1mple".to_string(),
            "ZywOo".to_string(),
            "NiKo".to_string(),
        ]
    }
    
    /// Analyze playstyle
    fn analyze_playstyle(&self, player: &PlayerState) -> String {
        if player.performance_metrics.accuracy > 0.8 {
            "Precision-focused rifler".to_string()
        } else if player.performance_metrics.positioning_score > 0.8 {
            "Tactical support player".to_string()
        } else {
            "Aggressive entry fragger".to_string()
        }
    }
    
    /// Suggest professional techniques
    fn suggest_pro_techniques(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "Counter-strafing for accurate shooting".to_string(),
            "Jiggle peeking for information gathering".to_string(),
            "Shoulder peeking to bait shots".to_string(),
            "Pop flashing for aggressive entries".to_string(),
        ]
    }
    
    /// Suggest aim training exercises
    fn suggest_aim_training(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "aim_botz - 15 minutes daily".to_string(),
            "Yprac maps for prefire practice".to_string(),
            "Aim Lab tracking exercises".to_string(),
            "Deathmatch with focus on crosshair placement".to_string(),
        ]
    }
    
    /// Suggest positioning improvements
    fn suggest_positioning_tips(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "Avoid standing in common angles".to_string(),
            "Use cover and fall back positions".to_string(),
            "Practice wide peeks vs. close peeks".to_string(),
            "Learn when to take map control".to_string(),
        ]
    }
    
    /// Suggest utility usage improvements
    fn suggest_utility_tips(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "Learn one-way smokes for this map".to_string(),
            "Practice pop flash setups with teammates".to_string(),
            "Save utility for retakes and executes".to_string(),
            "Use HE grenades for economic damage".to_string(),
        ]
    }
    
    /// Suggest economic improvements
    fn suggest_economic_advice(&self, _player: &PlayerState) -> Vec<String> {
        vec![
            "Force buy only when necessary".to_string(),
            "Drop weapons for teammates when possible".to_string(),
            "Buy utility even on eco rounds".to_string(),
            "Coordinate team buys for stronger rounds".to_string(),
        ]
    }
    
    /// Get current game state
    pub fn state(&self) -> &GameState {
        &self.state
    }
    
    /// Get entity manager
    pub fn entity_manager(&mut self) -> &mut EntityManager {
        &mut self.entity_manager
    }
    
    /// Get event manager
    pub fn event_manager(&mut self) -> &mut EventManager {
        &mut self.event_manager
    }
    
    /// Get state at specific tick
    pub fn get_state_at_tick(&self, tick: u32) -> Option<&GameState> {
        self.state_history
            .iter()
            .find(|(t, _)| *t == tick)
            .map(|(_, state)| state)
    }
    
    /// Get coaching insights for player
    pub fn get_coaching_insights(&self, user_id: u32) -> Option<&CoachingInsights> {
        self.coaching_cache.get(&user_id)
    }
    
    /// Get performance statistics
    pub fn get_performance_stats(&self) -> (u64, u64, f64) {
        let avg_processing_time = if self.update_count > 0 {
            self.processing_time_ms as f64 / self.update_count as f64
        } else {
            0.0
        };
        (self.update_count, self.processing_time_ms, avg_processing_time)
    }
}

impl Default for GameStateManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_game_state_manager_creation() {
        let manager = GameStateManager::new();
        assert_eq!(manager.state().current_tick, 0);
        assert_eq!(manager.state().round_number, 0);
    }
    
    #[test]
    fn test_state_update() {
        let mut manager = GameStateManager::new();
        
        manager.update_state(1000).unwrap();
        assert_eq!(manager.state().current_tick, 1000);
    }
    
    #[test]
    fn test_event_processing() {
        let mut manager = GameStateManager::new();
        
        let event = CS2GameEvent::RoundStart {
            time_limit: 115,
            frag_limit: 0,
            objective: "Bomb defusal".to_string(),
        };
        
        manager.process_event(1000, event).unwrap();
        assert_eq!(manager.state().round_number, 1);
        assert_eq!(manager.state().round_state.start_tick, 1000);
    }
    
    #[test]
    fn test_player_death_stats() {
        let mut manager = GameStateManager::new();
        
        // Add players
        manager.state.players.insert(1, PlayerState {
            user_id: 1,
            steam_id: 76561198000000001,
            name: "Player1".to_string(),
            team: Team::Terrorist,
            entity_id: None,
            health: 100,
            armor: 0,
            has_helmet: false,
            position: None,
            velocity: None,
            view_angles: None,
            active_weapon: None,
            weapons: Vec::new(),
            grenades: Vec::new(),
            money: 800,
            equipment_value: 0,
            round_stats: PlayerRoundStats {
                kills: 0, deaths: 0, assists: 0, damage_dealt: 0,
                damage_received: 0, shots_fired: 0, shots_hit: 0,
                headshots: 0, grenades_thrown: 0, money_spent: 0,
            },
            match_stats: PlayerMatchStats {
                total_kills: 0, total_deaths: 0, total_assists: 0,
                total_damage: 0, total_headshots: 0, adr: 0.0,
                kdr: 0.0, hsr: 0.0, rating: 1.0,
            },
            is_alive: true, is_connected: true, is_bot: false,
            is_spectating: false, is_defusing: false, is_planting: false,
            is_scoped: false, is_walking: false, is_ducking: false,
            performance_metrics: PlayerPerformanceMetrics {
                accuracy: 0.0, headshot_percentage: 0.0,
                crosshair_placement_score: 0.0, positioning_score: 0.0,
                decision_making_score: 0.0, utility_usage_score: 0.0,
                economy_management_score: 0.0, teamwork_score: 0.0,
                coaching_rating: 0.0, improvement_areas: Vec::new(),
            },
        });
        
        manager.state.players.insert(2, PlayerState {
            user_id: 2,
            steam_id: 76561198000000002,
            name: "Player2".to_string(),
            team: Team::CounterTerrorist,
            entity_id: None,
            health: 100,
            armor: 0,
            has_helmet: false,
            position: None,
            velocity: None,
            view_angles: None,
            active_weapon: None,
            weapons: Vec::new(),
            grenades: Vec::new(),
            money: 800,
            equipment_value: 0,
            round_stats: PlayerRoundStats {
                kills: 0, deaths: 0, assists: 0, damage_dealt: 0,
                damage_received: 0, shots_fired: 0, shots_hit: 0,
                headshots: 0, grenades_thrown: 0, money_spent: 0,
            },
            match_stats: PlayerMatchStats {
                total_kills: 0, total_deaths: 0, total_assists: 0,
                total_damage: 0, total_headshots: 0, adr: 0.0,
                kdr: 0.0, hsr: 0.0, rating: 1.0,
            },
            is_alive: true, is_connected: true, is_bot: false,
            is_spectating: false, is_defusing: false, is_planting: false,
            is_scoped: false, is_walking: false, is_ducking: false,
            performance_metrics: PlayerPerformanceMetrics {
                accuracy: 0.0, headshot_percentage: 0.0,
                crosshair_placement_score: 0.0, positioning_score: 0.0,
                decision_making_score: 0.0, utility_usage_score: 0.0,
                economy_management_score: 0.0, teamwork_score: 0.0,
                coaching_rating: 0.0, improvement_areas: Vec::new(),
            },
        });
        
        let death_event = CS2GameEvent::PlayerDeath {
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
            distance: 25.0,
        };
        
        manager.process_event(1100, death_event).unwrap();
        
        assert_eq!(manager.state.players[&1].round_stats.deaths, 1);
        assert_eq!(manager.state.players[&2].round_stats.kills, 1);
        assert_eq!(manager.state.players[&2].round_stats.headshots, 1);
        assert!(!manager.state.players[&1].is_alive);
    }
    
    #[test]
    fn test_coaching_insights_generation() {
        let mut manager = GameStateManager::new();
        
        // Add a player with some performance data
        manager.state.players.insert(1, PlayerState {
            user_id: 1,
            steam_id: 76561198000000001,
            name: "TestPlayer".to_string(),
            team: Team::Terrorist,
            entity_id: None,
            health: 100,
            armor: 0,
            has_helmet: false,
            position: None,
            velocity: None,
            view_angles: None,
            active_weapon: None,
            weapons: Vec::new(),
            grenades: Vec::new(),
            money: 800,
            equipment_value: 0,
            round_stats: PlayerRoundStats {
                kills: 0, deaths: 0, assists: 0, damage_dealt: 0,
                damage_received: 0, shots_fired: 0, shots_hit: 0,
                headshots: 0, grenades_thrown: 0, money_spent: 0,
            },
            match_stats: PlayerMatchStats {
                total_kills: 10, total_deaths: 8, total_assists: 5,
                total_damage: 1500, total_headshots: 3, adr: 75.0,
                kdr: 1.25, hsr: 0.3, rating: 1.1,
            },
            is_alive: true, is_connected: true, is_bot: false,
            is_spectating: false, is_defusing: false, is_planting: false,
            is_scoped: false, is_walking: false, is_ducking: false,
            performance_metrics: PlayerPerformanceMetrics {
                accuracy: 0.65, headshot_percentage: 0.45,
                crosshair_placement_score: 0.7, positioning_score: 0.8,
                decision_making_score: 0.6, utility_usage_score: 0.4,
                economy_management_score: 0.5, teamwork_score: 0.7,
                coaching_rating: 0.65, improvement_areas: Vec::new(),
            },
        });
        
        let insights = manager.generate_coaching_insights(1).unwrap();
        
        assert_eq!(insights.player_id, 1);
        assert!(!insights.strengths.is_empty());
        assert!(!insights.recommendations.is_empty());
        assert!(!insights.aim_training_suggestions.is_empty());
    }
    
    #[test]
    fn test_rank_estimation() {
        let manager = GameStateManager::new();
        
        let high_skill_player = PlayerState {
            user_id: 1,
            steam_id: 76561198000000001,
            name: "ProPlayer".to_string(),
            team: Team::Terrorist,
            entity_id: None,
            health: 100,
            armor: 0,
            has_helmet: false,
            position: None,
            velocity: None,
            view_angles: None,
            active_weapon: None,
            weapons: Vec::new(),
            grenades: Vec::new(),
            money: 800,
            equipment_value: 0,
            round_stats: PlayerRoundStats {
                kills: 0, deaths: 0, assists: 0, damage_dealt: 0,
                damage_received: 0, shots_fired: 0, shots_hit: 0,
                headshots: 0, grenades_thrown: 0, money_spent: 0,
            },
            match_stats: PlayerMatchStats {
                total_kills: 25, total_deaths: 10, total_assists: 8,
                total_damage: 3000, total_headshots: 15, adr: 150.0,
                kdr: 2.5, hsr: 0.6, rating: 1.5,
            },
            is_alive: true, is_connected: true, is_bot: false,
            is_spectating: false, is_defusing: false, is_planting: false,
            is_scoped: false, is_walking: false, is_ducking: false,
            performance_metrics: PlayerPerformanceMetrics {
                accuracy: 0.8, headshot_percentage: 0.6,
                crosshair_placement_score: 0.9, positioning_score: 0.85,
                decision_making_score: 0.8, utility_usage_score: 0.7,
                economy_management_score: 0.8, teamwork_score: 0.9,
                coaching_rating: 0.85, improvement_areas: Vec::new(),
            },
        };
        
        let rank = manager.estimate_rank(&high_skill_player);
        assert_eq!(rank, "Global Elite");
    }
}