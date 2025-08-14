use crate::BehavioralVector;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive feature extraction module for CS2 demo analysis
/// Implements feature extraction checklist from project requirements

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFeatures {
    pub player_mechanics: PlayerMechanicsFeatures,
    pub team_dynamics: TeamDynamicsFeatures,
    pub decision_metrics: DecisionMetricsFeatures,
    pub temporal_context: TemporalContextFeatures,
}

/// Player Mechanics Features - Individual player skill metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerMechanicsFeatures {
    // Aim Precision
    pub headshot_percentage: f32,
    pub headshot_percentage_per_weapon: HashMap<String, f32>,
    pub flick_accuracy: f32,
    pub flick_speed: f32,
    pub target_acquisition_time: f32,
    pub spray_control_deviation: f32,
    pub crosshair_placement_height: f32,
    pub pre_aim_accuracy: f32,

    // Movement
    pub counter_strafe_effectiveness: f32,
    pub peek_technique_score: f32, // wide vs tight peek analysis
    pub movement_efficiency: f32,
    pub position_transition_smoothness: f32,
    pub crouch_usage_pattern: f32,
    pub jump_usage_pattern: f32,
    pub air_strafe_control: f32,

    // Weapon Control
    pub recoil_control_consistency: f32,
    pub burst_vs_spray_preference: f32,
    pub weapon_switch_speed: f32,
    pub positioning_vs_weapon_range: f32,
    pub first_bullet_accuracy: f32,
    pub weapon_preference_patterns: HashMap<String, f32>,
}

/// Team Dynamics Features - Team coordination and positioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDynamicsFeatures {
    // Team Positioning
    pub formation_spread_vs_stack: f32,
    pub map_control_percentage: f32,
    pub defensive_setup_variations: f32,
    pub site_approach_patterns: HashMap<String, f32>,
    pub rotation_timing: f32,
    pub rotation_route_efficiency: f32,
    pub crossfire_setup_effectiveness: f32,

    // Utility Usage
    pub smoke_coverage_effectiveness: f32,
    pub flash_effectiveness_enemies: f32,
    pub flash_effectiveness_teammates: f32,
    pub molotov_area_denial_effectiveness: f32,
    pub grenade_damage_efficiency: f32,
    pub utility_timing_vs_executes: f32,
    pub support_utility_coordination: f32,

    // Tactical Execution
    pub execute_timing_consistency: f32,
    pub role_adherence: f32,
    pub trade_efficiency: f32,
    pub mid_round_adaptation_frequency: f32,
    pub default_strategy_identification: HashMap<String, f32>,
    pub execute_success_rate_by_type: HashMap<String, f32>,
}

/// Decision-Making Features - Strategic and tactical decision analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMetricsFeatures {
    // Economy Decisions
    pub buy_efficiency_value_per_dollar: f32,
    pub save_decision_quality: f32,
    pub force_buy_success_rate: f32,
    pub investment_utility_vs_weapons: f32,
    pub economic_impact_on_strategy: f32,

    // Decision Timing
    pub information_based_rotation_timing: f32,
    pub decision_speed_after_first_contact: f32,
    pub re_aggression_timing_patterns: f32,
    pub post_plant_positioning_decisions: f32,
    pub timeout_impact_on_decision_quality: f32,

    // Reaction Metrics
    pub reaction_time_visual_stimuli: f32,
    pub reaction_time_audio_stimuli: f32,
    pub adjustment_time_after_enemy_spotted: f32,
    pub reaction_consistency: f32,
    pub threat_prioritization_under_pressure: f32,
}

/// Temporal & Contextual Features - Round and situational analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContextFeatures {
    // Round Phase Context
    pub early_round_tendencies: HashMap<String, f32>,
    pub mid_round_adaptations: HashMap<String, f32>,
    pub late_round_decision_patterns: HashMap<String, f32>,
    pub clutch_performance_metrics: f32,

    // Map Context
    pub map_specific_tendencies: HashMap<String, f32>,
    pub position_preference_by_map: HashMap<String, HashMap<String, f32>>,
    pub success_rates_by_area: HashMap<String, f32>,
    pub route_preference_patterns: HashMap<String, f32>,

    // Opponent Adaptation
    pub counter_strategy_effectiveness: f32,
    pub adaptation_to_opponent_patterns: f32,
    pub anti_strategy_timing: f32,
    pub information_denial_effectiveness: f32,
}

/// Player Mechanics Extractor - Implements comprehensive aim and movement analysis
pub struct PlayerMechanicsExtractor {
    // Configuration for analysis sensitivity
    pub headshot_threshold: f32,
    pub flick_distance_threshold: f32,
    pub movement_smoothness_window: usize,
}

impl Default for PlayerMechanicsExtractor {
    fn default() -> Self {
        Self {
            headshot_threshold: 30.0, // degrees for headshot angle tolerance
            flick_distance_threshold: 45.0, // degrees for significant flick
            movement_smoothness_window: 10, // ticks for smoothness analysis
        }
    }
}

impl PlayerMechanicsExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract comprehensive player mechanics features from behavioral vectors
    pub fn extract_features(&self, vectors: &[BehavioralVector]) -> PlayerMechanicsFeatures {
        let mut features = PlayerMechanicsFeatures {
            headshot_percentage: 0.0,
            headshot_percentage_per_weapon: HashMap::new(),
            flick_accuracy: 0.0,
            flick_speed: 0.0,
            target_acquisition_time: 0.0,
            spray_control_deviation: 0.0,
            crosshair_placement_height: 0.0,
            pre_aim_accuracy: 0.0,
            counter_strafe_effectiveness: 0.0,
            peek_technique_score: 0.0,
            movement_efficiency: 0.0,
            position_transition_smoothness: 0.0,
            crouch_usage_pattern: 0.0,
            jump_usage_pattern: 0.0,
            air_strafe_control: 0.0,
            recoil_control_consistency: 0.0,
            burst_vs_spray_preference: 0.0,
            weapon_switch_speed: 0.0,
            positioning_vs_weapon_range: 0.0,
            first_bullet_accuracy: 0.0,
            weapon_preference_patterns: HashMap::new(),
        };

        if vectors.is_empty() {
            return features;
        }

        // Calculate aim precision metrics
        self.extract_aim_precision(&mut features, vectors);
        
        // Calculate movement metrics
        self.extract_movement_metrics(&mut features, vectors);
        
        // Calculate weapon control metrics
        self.extract_weapon_control(&mut features, vectors);

        features
    }

    fn extract_aim_precision(&self, features: &mut PlayerMechanicsFeatures, vectors: &[BehavioralVector]) {
        let mut total_yaw_changes: f32 = 0.0;
        let mut total_pitch_changes: f32 = 0.0;
        let mut flick_count = 0;
        let mut total_flick_accuracy = 0.0;
        let mut crosshair_heights: Vec<f32> = Vec::new();

        for window in vectors.windows(2) {
            let current = &window[0];
            let next = &window[1];

            let yaw_change = (next.yaw - current.yaw).abs();
            let pitch_change = (next.pitch - current.pitch).abs();

            total_yaw_changes += yaw_change;
            total_pitch_changes += pitch_change;

            // Detect flicks (large sudden aim changes)
            let total_change = (yaw_change * yaw_change + pitch_change * pitch_change).sqrt();
            if total_change > self.flick_distance_threshold {
                flick_count += 1;
                // Calculate flick accuracy (simplified - in real implementation would need hit data)
                let accuracy_estimate = 1.0 - (total_change / 180.0).min(1.0);
                total_flick_accuracy += accuracy_estimate;
            }

            // Collect crosshair height data (pitch values)
            crosshair_heights.push(current.pitch);
        }

        // Calculate averages and metrics
        if vectors.len() > 1 {
            features.spray_control_deviation = (total_yaw_changes + total_pitch_changes) / (vectors.len() - 1) as f32;
        }

        if flick_count > 0 {
            features.flick_accuracy = total_flick_accuracy / flick_count as f32;
            features.flick_speed = flick_count as f32 / vectors.len() as f32;
        }

        if !crosshair_heights.is_empty() {
            features.crosshair_placement_height = crosshair_heights.iter().sum::<f32>() / crosshair_heights.len() as f32;
        }

        // Simplified headshot percentage calculation (would need damage events in real implementation)
        features.headshot_percentage = 0.25; // Placeholder value
        features.pre_aim_accuracy = 0.7; // Placeholder value
        features.target_acquisition_time = 0.3; // Placeholder value in seconds
    }

    fn extract_movement_metrics(&self, features: &mut PlayerMechanicsFeatures, vectors: &[BehavioralVector]) {
        let mut total_velocity_changes = 0.0;
        let mut strafe_events = 0;
        let mut total_efficiency = 0.0;
        let mut air_time_ticks = 0;
        let mut total_air_strafe_quality = 0.0;

        for window in vectors.windows(2) {
            let current = &window[0];
            let next = &window[1];

            // Calculate velocity changes for movement efficiency
            let vel_change = ((next.vel_x - current.vel_x).powi(2) + 
                             (next.vel_y - current.vel_y).powi(2)).sqrt();
            total_velocity_changes += vel_change;

            // Detect strafing patterns
            if current.vel_x.abs() > 100.0 || current.vel_y.abs() > 100.0 {
                strafe_events += 1;
                let movement_vector_length = (current.vel_x.powi(2) + current.vel_y.powi(2)).sqrt();
                if movement_vector_length > 0.0 {
                    total_efficiency += movement_vector_length / 320.0; // Normalize by max movement speed
                }
            }

            // Air strafe analysis
            if current.is_airborne > 0.5 {
                air_time_ticks += 1;
                // Calculate air strafe quality based on velocity direction vs input
                let strafe_quality = self.calculate_air_strafe_quality(current);
                total_air_strafe_quality += strafe_quality;
            }
        }

        // Calculate movement metrics
        if strafe_events > 0 {
            features.movement_efficiency = total_efficiency / strafe_events as f32;
            features.counter_strafe_effectiveness = self.estimate_counter_strafe_effectiveness(vectors);
        }

        if air_time_ticks > 0 {
            features.air_strafe_control = total_air_strafe_quality / air_time_ticks as f32;
        }

        // Calculate position transition smoothness
        features.position_transition_smoothness = self.calculate_movement_smoothness(vectors);
        
        // Usage patterns (simplified)
        features.crouch_usage_pattern = 0.1; // Placeholder
        features.jump_usage_pattern = 0.05; // Placeholder  
        features.peek_technique_score = 0.7; // Placeholder
    }

    fn extract_weapon_control(&self, features: &mut PlayerMechanicsFeatures, vectors: &[BehavioralVector]) {
        let mut weapon_usage: HashMap<u16, usize> = HashMap::new();
        let mut recoil_consistency_sum = 0.0;
        let mut weapon_switches = 0;
        let mut burst_shots = 0;
        let mut spray_shots = 0;

        for window in vectors.windows(2) {
            let current = &window[0];
            let next = &window[1];

            // Track weapon usage
            *weapon_usage.entry(current.weapon_id).or_insert(0) += 1;

            // Detect weapon switches
            if current.weapon_id != next.weapon_id {
                weapon_switches += 1;
            }

            // Analyze recoil control (simplified)
            let recoil_control = self.calculate_recoil_control_quality(current, next);
            recoil_consistency_sum += recoil_control;

            // Classify as burst or spray based on ammo usage patterns
            if current.ammo > next.ammo {
                let shots_fired = current.ammo - next.ammo;
                if shots_fired <= 3.0 {
                    burst_shots += shots_fired as i32;
                } else {
                    spray_shots += shots_fired as i32;
                }
            }
        }

        // Calculate weapon control metrics
        if vectors.len() > 1 {
            features.recoil_control_consistency = recoil_consistency_sum / (vectors.len() - 1) as f32;
            features.weapon_switch_speed = weapon_switches as f32 / vectors.len() as f32;
        }

        if burst_shots + spray_shots > 0 {
            features.burst_vs_spray_preference = burst_shots as f32 / (burst_shots + spray_shots) as f32;
        }

        // Convert weapon usage to preference patterns
        let total_usage: usize = weapon_usage.values().sum();
        for (weapon_id, usage) in weapon_usage {
            let weapon_name = self.weapon_id_to_name(weapon_id);
            features.weapon_preference_patterns.insert(
                weapon_name, 
                usage as f32 / total_usage as f32
            );
        }

        // Placeholder values for complex metrics
        features.positioning_vs_weapon_range = 0.8;
        features.first_bullet_accuracy = 0.75;
    }

    fn calculate_air_strafe_quality(&self, vector: &BehavioralVector) -> f32 {
        // Simplified air strafe quality calculation
        // In real implementation, would analyze mouse movement vs velocity direction
        let velocity_magnitude = (vector.vel_x.powi(2) + vector.vel_y.powi(2)).sqrt();
        (velocity_magnitude / 320.0).min(1.0) // Normalize and cap at 1.0
    }

    fn estimate_counter_strafe_effectiveness(&self, vectors: &[BehavioralVector]) -> f32 {
        // Simplified counter-strafe detection
        // Would analyze velocity reduction patterns in real implementation
        0.8 // Placeholder value
    }

    fn calculate_movement_smoothness(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < self.movement_smoothness_window {
            return 1.0;
        }

        let mut smoothness_sum = 0.0;
        let mut window_count = 0;

        for window in vectors.windows(self.movement_smoothness_window) {
            let mut acceleration_changes = 0.0;
            
            for i in 1..window.len()-1 {
                let prev_vel = (window[i-1].vel_x.powi(2) + window[i-1].vel_y.powi(2)).sqrt();
                let curr_vel = (window[i].vel_x.powi(2) + window[i].vel_y.powi(2)).sqrt();
                let next_vel = (window[i+1].vel_x.powi(2) + window[i+1].vel_y.powi(2)).sqrt();
                
                let prev_accel = curr_vel - prev_vel;
                let curr_accel = next_vel - curr_vel;
                
                acceleration_changes += (curr_accel - prev_accel).abs();
            }
            
            // Lower acceleration changes indicate smoother movement
            let smoothness = 1.0 / (1.0 + acceleration_changes / 100.0);
            smoothness_sum += smoothness;
            window_count += 1;
        }

        if window_count > 0 {
            smoothness_sum / window_count as f32
        } else {
            1.0
        }
    }

    fn calculate_recoil_control_quality(&self, current: &BehavioralVector, next: &BehavioralVector) -> f32 {
        // Simplified recoil control calculation
        // In real implementation, would compare aim adjustments to weapon recoil patterns
        let yaw_change = (next.yaw - current.yaw).abs();
        let pitch_change = (next.pitch - current.pitch).abs();
        let total_change = (yaw_change.powi(2) + pitch_change.powi(2)).sqrt();
        
        // Good recoil control = moderate, consistent aim adjustments
        if total_change > 0.0 && total_change < 5.0 {
            1.0 - (total_change / 5.0)
        } else {
            0.0
        }
    }

    fn weapon_id_to_name(&self, weapon_id: u16) -> String {
        // Simplified weapon ID to name mapping
        match weapon_id {
            7 => "AK-47".to_string(),
            16 => "M4A4".to_string(),
            60 => "M4A1-S".to_string(),
            40 => "AWP".to_string(),
            _ => format!("weapon_{}", weapon_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_mechanics_extractor() {
        let extractor = PlayerMechanicsExtractor::new();
        
        // Create test vectors with some movement and aim data
        let vectors = vec![
            BehavioralVector {
                tick: 1,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 100.0,
                pos_x: 0.0,
                pos_y: 0.0,
                pos_z: 64.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                weapon_id: 7,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 0.0,
                delta_pitch: 0.0,
            },
            BehavioralVector {
                tick: 2,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 100.0,
                pos_x: 10.0,
                pos_y: 0.0,
                pos_z: 64.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 5.0,
                pitch: -2.0,
                weapon_id: 7,
                ammo: 27.0,
                is_airborne: 0.0,
                delta_yaw: 5.0,
                delta_pitch: -2.0,
            },
        ];

        let features = extractor.extract_features(&vectors);
        
        // Basic validation
        assert!(features.headshot_percentage >= 0.0);
        assert!(features.movement_efficiency >= 0.0);
        assert!(features.recoil_control_consistency >= 0.0);
        assert!(!features.weapon_preference_patterns.is_empty());
    }

    #[test]
    fn test_empty_vectors() {
        let extractor = PlayerMechanicsExtractor::new();
        let features = extractor.extract_features(&[]);
        
        assert_eq!(features.headshot_percentage, 0.0);
        assert_eq!(features.movement_efficiency, 0.0);
    }

    #[test]
    fn test_movement_smoothness_calculation() {
        let extractor = PlayerMechanicsExtractor::new();
        
        // Create vectors with smooth movement
        let smooth_vectors: Vec<BehavioralVector> = (0..20)
            .map(|i| BehavioralVector {
                tick: i,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 100.0,
                pos_x: i as f32 * 10.0,
                pos_y: 0.0,
                pos_z: 64.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 0.0,
                pitch: 0.0,
                weapon_id: 7,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 0.0,
                delta_pitch: 0.0,
            })
            .collect();

        let smoothness = extractor.calculate_movement_smoothness(&smooth_vectors);
        assert!(smoothness > 0.8); // Should be high for smooth movement
    }
}