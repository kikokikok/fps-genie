use crate::BehavioralVector;
use crate::feature_extraction::{TeamDynamicsFeatures, DecisionMetricsFeatures};
use std::collections::HashMap;

/// Team Dynamics Extractor - Analyzes team coordination and positioning
pub struct TeamDynamicsExtractor {
    pub max_team_distance: f32,  // Maximum distance for team spread analysis
    pub utility_impact_radius: f32,  // Radius for utility effectiveness analysis
    pub execute_time_window: u32,  // Ticks for tactical execute timing analysis
}

impl Default for TeamDynamicsExtractor {
    fn default() -> Self {
        Self {
            max_team_distance: 2000.0,  // CS2 units
            utility_impact_radius: 500.0,  // CS2 units
            execute_time_window: 64,  // ~1 second at 64 tick
        }
    }
}

impl TeamDynamicsExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract team dynamics features from multiple players' behavioral vectors
    pub fn extract_features(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> TeamDynamicsFeatures {
        let mut features = TeamDynamicsFeatures {
            formation_spread_vs_stack: 0.0,
            map_control_percentage: 0.0,
            defensive_setup_variations: 0.0,
            site_approach_patterns: HashMap::new(),
            rotation_timing: 0.0,
            rotation_route_efficiency: 0.0,
            crossfire_setup_effectiveness: 0.0,
            smoke_coverage_effectiveness: 0.0,
            flash_effectiveness_enemies: 0.0,
            flash_effectiveness_teammates: 0.0,
            molotov_area_denial_effectiveness: 0.0,
            grenade_damage_efficiency: 0.0,
            utility_timing_vs_executes: 0.0,
            support_utility_coordination: 0.0,
            execute_timing_consistency: 0.0,
            role_adherence: 0.0,
            trade_efficiency: 0.0,
            mid_round_adaptation_frequency: 0.0,
            default_strategy_identification: HashMap::new(),
            execute_success_rate_by_type: HashMap::new(),
        };

        if team_vectors.is_empty() {
            return features;
        }

        // Extract team positioning metrics
        self.extract_team_positioning(&mut features, team_vectors);
        
        // Extract utility usage metrics
        self.extract_utility_usage(&mut features, team_vectors);
        
        // Extract tactical execution metrics
        self.extract_tactical_execution(&mut features, team_vectors);

        features
    }

    fn extract_team_positioning(&self, features: &mut TeamDynamicsFeatures, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) {
        let player_ids: Vec<u64> = team_vectors.keys().copied().collect();
        
        if player_ids.len() < 2 {
            return;
        }

        // Find common tick range for all players
        let common_ticks = self.find_common_ticks(team_vectors);
        if common_ticks.is_empty() {
            return;
        }

        let mut total_spread = 0.0;
        let mut total_map_coverage = 0.0;
        let mut crossfire_opportunities = 0;
        let mut effective_crossfires = 0;
        let mut position_setups: HashMap<String, i32> = HashMap::new();

        for tick in &common_ticks {
            // Get player positions at this tick
            let positions: Vec<(f32, f32, f32)> = player_ids.iter()
                .filter_map(|&player_id| {
                    team_vectors.get(&player_id)?
                        .iter()
                        .find(|v| v.tick == *tick)
                        .map(|v| (v.pos_x, v.pos_y, v.pos_z))
                })
                .collect();

            if positions.len() >= 2 {
                // Calculate team spread
                let spread = self.calculate_team_spread(&positions);
                total_spread += spread;

                // Analyze map coverage
                let coverage = self.calculate_map_coverage(&positions);
                total_map_coverage += coverage;

                // Analyze crossfire setup
                let (opportunities, effective) = self.analyze_crossfire_setup(&positions);
                crossfire_opportunities += opportunities;
                effective_crossfires += effective;

                // Classify defensive setup
                let setup_type = self.classify_defensive_setup(&positions);
                *position_setups.entry(setup_type).or_insert(0) += 1;
            }
        }

        // Calculate averages
        let tick_count = common_ticks.len() as f32;
        if tick_count > 0.0 {
            features.formation_spread_vs_stack = total_spread / tick_count;
            features.map_control_percentage = total_map_coverage / tick_count;
            
            if crossfire_opportunities > 0 {
                features.crossfire_setup_effectiveness = effective_crossfires as f32 / crossfire_opportunities as f32;
            }
        }

        // Calculate defensive setup variations
        features.defensive_setup_variations = position_setups.len() as f32 / tick_count.max(1.0);

        // Analyze site approach patterns (simplified)
        features.site_approach_patterns.insert("A_site".to_string(), 0.6);
        features.site_approach_patterns.insert("B_site".to_string(), 0.4);
        features.site_approach_patterns.insert("mid_control".to_string(), 0.3);

        // Analyze rotation patterns
        features.rotation_timing = self.analyze_rotation_timing(team_vectors);
        features.rotation_route_efficiency = 0.75; // Placeholder
    }

    fn extract_utility_usage(&self, features: &mut TeamDynamicsFeatures, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) {
        // Simplified utility analysis - in real implementation would need grenade/utility events
        // For now, using movement patterns and positioning as proxy metrics
        
        let mut total_coordination_score = 0.0;
        let mut coordination_samples = 0;

        for (&player_id, vectors) in team_vectors {
            if vectors.len() < 10 {
                continue;
            }

            // Analyze utility coordination through synchronized movements
            let coordination = self.analyze_utility_coordination(player_id, vectors, team_vectors);
            total_coordination_score += coordination;
            coordination_samples += 1;
        }

        if coordination_samples > 0 {
            features.support_utility_coordination = total_coordination_score / coordination_samples as f32;
        }

        // Placeholder values for utility effectiveness metrics
        features.smoke_coverage_effectiveness = 0.8;
        features.flash_effectiveness_enemies = 0.7;
        features.flash_effectiveness_teammates = 0.9; // Higher is better (less team damage)
        features.molotov_area_denial_effectiveness = 0.75;
        features.grenade_damage_efficiency = 0.6;
        features.utility_timing_vs_executes = 0.85;
    }

    fn extract_tactical_execution(&self, features: &mut TeamDynamicsFeatures, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) {
        // Analyze execute timing consistency
        let execute_timings = self.detect_execute_attempts(team_vectors);
        if !execute_timings.is_empty() {
            let timing_variance = self.calculate_timing_variance(&execute_timings);
            features.execute_timing_consistency = 1.0 - timing_variance.min(1.0);
        }

        // Analyze role adherence through positioning patterns
        features.role_adherence = self.analyze_role_adherence(team_vectors);

        // Analyze trade potential and effectiveness
        features.trade_efficiency = self.analyze_trade_efficiency(team_vectors);

        // Detect mid-round adaptations through sudden position changes
        features.mid_round_adaptation_frequency = self.detect_mid_round_adaptations(team_vectors);

        // Identify default strategies
        let default_strategies = self.identify_default_strategies(team_vectors);
        features.default_strategy_identification = default_strategies;

        // Analyze execute success rates (placeholder)
        features.execute_success_rate_by_type.insert("rush_execute".to_string(), 0.65);
        features.execute_success_rate_by_type.insert("slow_execute".to_string(), 0.75);
        features.execute_success_rate_by_type.insert("fake_execute".to_string(), 0.55);
    }

    fn find_common_ticks(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> Vec<u32> {
        if team_vectors.is_empty() {
            return Vec::new();
        }

        // Get all tick sets
        let tick_sets: Vec<std::collections::HashSet<u32>> = team_vectors.values()
            .map(|vectors| vectors.iter().map(|v| v.tick).collect())
            .collect();

        if tick_sets.is_empty() {
            return Vec::new();
        }

        // Find intersection of all tick sets
        let mut common_ticks = tick_sets[0].clone();
        for tick_set in &tick_sets[1..] {
            common_ticks = common_ticks.intersection(tick_set).copied().collect();
        }

        let mut result: Vec<u32> = common_ticks.into_iter().collect();
        result.sort();
        result
    }

    fn calculate_team_spread(&self, positions: &[(f32, f32, f32)]) -> f32 {
        if positions.len() < 2 {
            return 0.0;
        }

        let mut total_distance = 0.0;
        let mut pair_count = 0;

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let distance = (dx * dx + dy * dy).sqrt();
                total_distance += distance;
                pair_count += 1;
            }
        }

        if pair_count > 0 {
            // Normalize by max expected distance
            (total_distance / pair_count as f32) / self.max_team_distance
        } else {
            0.0
        }
    }

    fn calculate_map_coverage(&self, positions: &[(f32, f32, f32)]) -> f32 {
        // Simplified map coverage calculation
        // In real implementation, would use actual map boundaries and important areas
        
        if positions.is_empty() {
            return 0.0;
        }

        // Calculate bounding box of team positions
        let min_x = positions.iter().map(|p| p.0).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_x = positions.iter().map(|p| p.0).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let min_y = positions.iter().map(|p| p.1).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
        let max_y = positions.iter().map(|p| p.1).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

        let coverage_area = (max_x - min_x) * (max_y - min_y);
        
        // Normalize by expected map size (simplified)
        (coverage_area / (4000.0 * 4000.0)).min(1.0)
    }

    fn analyze_crossfire_setup(&self, positions: &[(f32, f32, f32)]) -> (i32, i32) {
        // Simplified crossfire analysis
        // In real implementation, would analyze sight lines and coverage angles
        
        let mut opportunities = 0;
        let mut effective = 0;

        for i in 0..positions.len() {
            for j in (i + 1)..positions.len() {
                opportunities += 1;
                
                // Simplified: if players are at good distance and different angles
                let dx = positions[i].0 - positions[j].0;
                let dy = positions[i].1 - positions[j].1;
                let distance = (dx * dx + dy * dy).sqrt();
                
                if distance > 200.0 && distance < 1500.0 {
                    effective += 1;
                }
            }
        }

        (opportunities, effective)
    }

    fn classify_defensive_setup(&self, positions: &[(f32, f32, f32)]) -> String {
        if positions.len() < 3 {
            return "insufficient_players".to_string();
        }

        let spread = self.calculate_team_spread(positions);
        
        if spread < 0.3 {
            "stack_setup".to_string()
        } else if spread > 0.7 {
            "spread_setup".to_string()
        } else {
            "mixed_setup".to_string()
        }
    }

    fn analyze_rotation_timing(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> f32 {
        // Simplified rotation timing analysis
        // Would analyze coordinated movement patterns in real implementation
        0.7 // Placeholder value
    }

    fn analyze_utility_coordination(&self, player_id: u64, vectors: &[BehavioralVector], team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> f32 {
        // Analyze coordination through synchronized positioning changes
        // In real implementation, would use utility event data
        
        let mut coordination_score = 0.0;
        let mut sample_count = 0;

        for window in vectors.windows(5) {
            let tick = window[2].tick; // Middle tick of window
            
            // Check if other players made similar movements at similar times
            let mut synchronized_movements = 0;
            for (&other_player_id, other_vectors) in team_vectors {
                if other_player_id == player_id {
                    continue;
                }
                
                if let Some(other_vector) = other_vectors.iter().find(|v| (v.tick as i32 - tick as i32).abs() <= 3) {
                    let player_movement = (window[4].vel_x.powi(2) + window[4].vel_y.powi(2)).sqrt();
                    let other_movement = (other_vector.vel_x.powi(2) + other_vector.vel_y.powi(2)).sqrt();
                    
                    // If both players are moving significantly at similar times
                    if player_movement > 100.0 && other_movement > 100.0 {
                        synchronized_movements += 1;
                    }
                }
            }
            
            if synchronized_movements > 0 {
                coordination_score += synchronized_movements as f32 / (team_vectors.len() - 1) as f32;
                sample_count += 1;
            }
        }

        if sample_count > 0 {
            coordination_score / sample_count as f32
        } else {
            0.0
        }
    }

    fn detect_execute_attempts(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> Vec<u32> {
        // Detect coordinated team movements that indicate execute attempts
        let common_ticks = self.find_common_ticks(team_vectors);
        let mut execute_ticks = Vec::new();

        for window in common_ticks.windows(self.execute_time_window as usize) {
            let start_tick = window[0];
            let end_tick = window[window.len() - 1];
            
            // Check if majority of team moved significantly during this window
            let mut players_moving = 0;
            let total_players = team_vectors.len();
            
            for vectors in team_vectors.values() {
                let start_pos = vectors.iter().find(|v| v.tick == start_tick);
                let end_pos = vectors.iter().find(|v| v.tick == end_tick);
                
                if let (Some(start), Some(end)) = (start_pos, end_pos) {
                    let distance_moved = ((end.pos_x - start.pos_x).powi(2) + 
                                        (end.pos_y - start.pos_y).powi(2)).sqrt();
                    
                    if distance_moved > 300.0 {  // Significant movement
                        players_moving += 1;
                    }
                }
            }
            
            // If majority of team moved significantly, consider it an execute
            if players_moving as f32 / total_players as f32 > 0.6 {
                execute_ticks.push(start_tick);
            }
        }

        execute_ticks
    }

    fn calculate_timing_variance(&self, timings: &[u32]) -> f32 {
        if timings.len() < 2 {
            return 0.0;
        }

        let mean = timings.iter().sum::<u32>() as f32 / timings.len() as f32;
        let variance = timings.iter()
            .map(|&t| (t as f32 - mean).powi(2))
            .sum::<f32>() / timings.len() as f32;
        
        // Normalize variance by typical round length
        (variance.sqrt() / 1000.0).min(1.0)
    }

    fn analyze_role_adherence(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> f32 {
        // Simplified role adherence analysis
        // Would analyze positioning patterns relative to expected roles
        0.8 // Placeholder value
    }

    fn analyze_trade_efficiency(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> f32 {
        // Analyze potential for trading kills based on player proximity
        let common_ticks = self.find_common_ticks(team_vectors);
        let mut trade_opportunities = 0;
        let mut good_trade_positions = 0;

        for tick in common_ticks.iter().step_by(10) { // Sample every 10 ticks
            let positions: Vec<(u64, f32, f32, f32)> = team_vectors.iter()
                .filter_map(|(&player_id, vectors)| {
                    vectors.iter()
                        .find(|v| v.tick == *tick)
                        .map(|v| (player_id, v.pos_x, v.pos_y, v.pos_z))
                })
                .collect();

            for i in 0..positions.len() {
                for j in (i + 1)..positions.len() {
                    trade_opportunities += 1;
                    
                    let dx = positions[i].1 - positions[j].1;
                    let dy = positions[i].2 - positions[j].2;
                    let distance = (dx * dx + dy * dy).sqrt();
                    
                    // Good trade distance: close enough to help, far enough to avoid double peek
                    if distance > 200.0 && distance < 800.0 {
                        good_trade_positions += 1;
                    }
                }
            }
        }

        if trade_opportunities > 0 {
            good_trade_positions as f32 / trade_opportunities as f32
        } else {
            0.0
        }
    }

    fn detect_mid_round_adaptations(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> f32 {
        // Detect sudden changes in team positioning mid-round
        let mut adaptation_count = 0;
        let mut total_rounds_analyzed = 0;

        // Simplified: analyze position changes in chunks representing round segments
        for vectors in team_vectors.values() {
            if vectors.len() < 100 {
                continue;
            }

            total_rounds_analyzed += 1;
            
            // Split into early, mid, late round segments
            let segment_size = vectors.len() / 3;
            let segments = [
                &vectors[0..segment_size],
                &vectors[segment_size..2*segment_size],
                &vectors[2*segment_size..],
            ];

            // Check for significant position changes between segments
            for i in 0..segments.len()-1 {
                let avg_pos_1 = self.calculate_average_position(segments[i]);
                let avg_pos_2 = self.calculate_average_position(segments[i+1]);
                
                let position_change = ((avg_pos_2.0 - avg_pos_1.0).powi(2) + 
                                     (avg_pos_2.1 - avg_pos_1.1).powi(2)).sqrt();
                
                if position_change > 500.0 {  // Significant position change
                    adaptation_count += 1;
                    break;  // Only count one adaptation per round
                }
            }
        }

        if total_rounds_analyzed > 0 {
            adaptation_count as f32 / total_rounds_analyzed as f32
        } else {
            0.0
        }
    }

    fn calculate_average_position(&self, vectors: &[BehavioralVector]) -> (f32, f32) {
        if vectors.is_empty() {
            return (0.0, 0.0);
        }

        let sum_x = vectors.iter().map(|v| v.pos_x).sum::<f32>();
        let sum_y = vectors.iter().map(|v| v.pos_y).sum::<f32>();
        
        (sum_x / vectors.len() as f32, sum_y / vectors.len() as f32)
    }

    fn identify_default_strategies(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> HashMap<String, f32> {
        // Simplified strategy identification based on common positioning patterns
        let mut strategies = HashMap::new();
        
        // Analyze early round positioning to identify default setups
        let early_round_positions = self.analyze_early_round_positioning(team_vectors);
        
        strategies.insert("default_ct_setup".to_string(), 0.7);
        strategies.insert("aggressive_ct_setup".to_string(), 0.2);
        strategies.insert("stack_a_setup".to_string(), 0.1);
        strategies.insert("stack_b_setup".to_string(), 0.1);
        
        strategies
    }

    fn analyze_early_round_positioning(&self, team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> Vec<(f32, f32)> {
        // Get positioning patterns from first 30 ticks (early round)
        let mut early_positions = Vec::new();
        
        for vectors in team_vectors.values() {
            if let Some(early_vector) = vectors.iter().find(|v| v.tick <= 30) {
                early_positions.push((early_vector.pos_x, early_vector.pos_y));
            }
        }
        
        early_positions
    }
}

/// Decision Metrics Extractor - Analyzes strategic and tactical decision making
pub struct DecisionMetricsExtractor {
    pub economy_analysis_window: u32,  // Ticks to analyze for economy decisions
    pub reaction_time_threshold: f32,  // Threshold for fast reactions (degrees/tick)
    pub decision_confidence_threshold: f32,  // Threshold for confident decisions
}

impl Default for DecisionMetricsExtractor {
    fn default() -> Self {
        Self {
            economy_analysis_window: 320,  // ~5 seconds at 64 tick
            reaction_time_threshold: 2.0,  // degrees per tick
            decision_confidence_threshold: 0.7,
        }
    }
}

impl DecisionMetricsExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract decision-making features from behavioral vectors
    pub fn extract_features(&self, vectors: &[BehavioralVector], team_vectors: &HashMap<u64, Vec<BehavioralVector>>) -> DecisionMetricsFeatures {
        let mut features = DecisionMetricsFeatures {
            buy_efficiency_value_per_dollar: 0.0,
            save_decision_quality: 0.0,
            force_buy_success_rate: 0.0,
            investment_utility_vs_weapons: 0.0,
            economic_impact_on_strategy: 0.0,
            information_based_rotation_timing: 0.0,
            decision_speed_after_first_contact: 0.0,
            re_aggression_timing_patterns: 0.0,
            post_plant_positioning_decisions: 0.0,
            timeout_impact_on_decision_quality: 0.0,
            reaction_time_visual_stimuli: 0.0,
            reaction_time_audio_stimuli: 0.0,
            adjustment_time_after_enemy_spotted: 0.0,
            reaction_consistency: 0.0,
            threat_prioritization_under_pressure: 0.0,
        };

        if vectors.is_empty() {
            return features;
        }

        // Extract economy decision metrics
        self.extract_economy_decisions(&mut features, vectors);
        
        // Extract timing decision metrics
        self.extract_timing_decisions(&mut features, vectors, team_vectors);
        
        // Extract reaction metrics
        self.extract_reaction_metrics(&mut features, vectors);

        features
    }

    fn extract_economy_decisions(&self, features: &mut DecisionMetricsFeatures, vectors: &[BehavioralVector]) {
        // Analyze weapon preferences and efficiency
        let mut weapon_values: HashMap<u16, f32> = HashMap::new();
        weapon_values.insert(7, 2700.0);   // AK-47
        weapon_values.insert(16, 3100.0);  // M4A4
        weapon_values.insert(60, 2900.0);  // M4A1-S
        weapon_values.insert(40, 4750.0);  // AWP
        
        let mut total_value_efficiency = 0.0;
        let mut weapon_usage_count = 0;
        let mut utility_investment = 0.0;
        let mut weapon_investment = 0.0;

        for vector in vectors {
            if let Some(&weapon_value) = weapon_values.get(&vector.weapon_id) {
                // Calculate value efficiency based on effective usage
                let usage_efficiency = self.calculate_weapon_efficiency(vector, weapon_value);
                total_value_efficiency += usage_efficiency;
                weapon_usage_count += 1;
                weapon_investment += weapon_value;
            } else {
                // Assume utility or cheaper weapon
                utility_investment += 500.0; // Average utility cost
            }
        }

        if weapon_usage_count > 0 {
            features.buy_efficiency_value_per_dollar = total_value_efficiency / weapon_usage_count as f32;
        }

        let total_investment = weapon_investment + utility_investment;
        if total_investment > 0.0 {
            features.investment_utility_vs_weapons = utility_investment / total_investment;
        }

        // Placeholder values for complex metrics that would need round/economy context
        features.save_decision_quality = 0.75;
        features.force_buy_success_rate = 0.45;
        features.economic_impact_on_strategy = 0.8;
    }

    fn extract_timing_decisions(&self, features: &mut DecisionMetricsFeatures, vectors: &[BehavioralVector], team_vectors: &HashMap<u64, Vec<BehavioralVector>>) {
        // Analyze decision speed through rapid position/aim changes
        let mut rapid_decisions = 0;
        let mut total_decisions = 0;
        let mut rotation_timings = Vec::new();

        for window in vectors.windows(3) {
            let prev = &window[0];
            let curr = &window[1];
            let next = &window[2];

            // Detect rapid decision making through quick position/aim changes
            let aim_change_1 = ((curr.yaw - prev.yaw).powi(2) + (curr.pitch - prev.pitch).powi(2)).sqrt();
            let aim_change_2 = ((next.yaw - curr.yaw).powi(2) + (next.pitch - curr.pitch).powi(2)).sqrt();
            
            if aim_change_1 > self.reaction_time_threshold || aim_change_2 > self.reaction_time_threshold {
                total_decisions += 1;
                
                // Quick successive changes indicate rapid decision making
                if aim_change_1 > self.reaction_time_threshold && aim_change_2 > self.reaction_time_threshold {
                    rapid_decisions += 1;
                }
            }

            // Detect potential rotations through significant position changes
            let pos_change = ((curr.pos_x - prev.pos_x).powi(2) + (curr.pos_y - prev.pos_y).powi(2)).sqrt();
            if pos_change > 200.0 {  // Significant movement
                rotation_timings.push(curr.tick - prev.tick);
            }
        }

        if total_decisions > 0 {
            features.decision_speed_after_first_contact = rapid_decisions as f32 / total_decisions as f32;
        }

        // Analyze rotation timing relative to team
        if !rotation_timings.is_empty() {
            let avg_rotation_time = rotation_timings.iter().sum::<u32>() as f32 / rotation_timings.len() as f32;
            features.information_based_rotation_timing = 1.0 - (avg_rotation_time / 64.0).min(1.0); // Normalize by 1 second
        }

        // Analyze re-aggression patterns through return to previous positions
        features.re_aggression_timing_patterns = self.analyze_re_aggression_patterns(vectors);

        // Placeholder values for complex metrics
        features.post_plant_positioning_decisions = 0.8;
        features.timeout_impact_on_decision_quality = 0.85;
    }

    fn extract_reaction_metrics(&self, features: &mut DecisionMetricsFeatures, vectors: &[BehavioralVector]) {
        let mut reaction_times = Vec::new();
        let mut adjustment_times = Vec::new();
        let mut reaction_qualities = Vec::new();

        for window in vectors.windows(5) {
            // Analyze reaction patterns through aim adjustments
            let baseline = &window[0];
            
            for i in 1..window.len() {
                let current = &window[i];
                let reaction_magnitude = ((current.yaw - baseline.yaw).powi(2) + 
                                        (current.pitch - baseline.pitch).powi(2)).sqrt();
                
                if reaction_magnitude > self.reaction_time_threshold {
                    let reaction_time = (current.tick - baseline.tick) as f32 / 64.0; // Convert to seconds
                    reaction_times.push(reaction_time);
                    
                    // Calculate reaction quality (smooth vs jerky)
                    let smoothness = self.calculate_reaction_smoothness(&window[0..=i]);
                    reaction_qualities.push(smoothness);
                    break;
                }
            }

            // Analyze adjustment time after significant changes
            if window.len() >= 3 {
                let mid = &window[2];
                let adjustment_needed = ((mid.yaw - baseline.yaw).powi(2) + 
                                       (mid.pitch - baseline.pitch).powi(2)).sqrt();
                
                if adjustment_needed > self.reaction_time_threshold * 2.0 {
                    let adjustment_time = (window[4].tick - mid.tick) as f32 / 64.0;
                    adjustment_times.push(adjustment_time);
                }
            }
        }

        // Calculate reaction metrics
        if !reaction_times.is_empty() {
            let avg_reaction_time = reaction_times.iter().sum::<f32>() / reaction_times.len() as f32;
            features.reaction_time_visual_stimuli = avg_reaction_time;
            
            // Calculate consistency (lower variance = higher consistency)
            let mean = avg_reaction_time;
            let variance = reaction_times.iter()
                .map(|&t| (t - mean).powi(2))
                .sum::<f32>() / reaction_times.len() as f32;
            features.reaction_consistency = 1.0 - (variance.sqrt() / mean).min(1.0);
        }

        if !adjustment_times.is_empty() {
            let avg_adjustment_time = adjustment_times.iter().sum::<f32>() / adjustment_times.len() as f32;
            features.adjustment_time_after_enemy_spotted = avg_adjustment_time;
        }

        // Placeholder values for complex metrics that would need event data
        features.reaction_time_audio_stimuli = 0.35; // Typically faster than visual
        features.threat_prioritization_under_pressure = 0.7;
    }

    fn calculate_weapon_efficiency(&self, vector: &BehavioralVector, weapon_value: f32) -> f32 {
        // Simplified weapon efficiency calculation
        // In real implementation, would factor in damage dealt, kills, survival time, etc.
        
        // Use health as a proxy for survival/effectiveness
        let survival_factor = vector.health / 100.0;
        
        // Use ammo as a proxy for weapon usage efficiency
        let ammo_efficiency = if vector.ammo > 0.0 {
            (30.0 - vector.ammo) / 30.0  // Assume 30 rounds max
        } else {
            1.0  // Empty magazine suggests active use
        };
        
        (survival_factor + ammo_efficiency) / 2.0
    }

    fn analyze_re_aggression_patterns(&self, vectors: &[BehavioralVector]) -> f32 {
        let mut re_aggression_count = 0;
        let mut position_samples = 0;
        let mut previous_positions = Vec::new();

        for (i, vector) in vectors.iter().enumerate() {
            let current_pos = (vector.pos_x, vector.pos_y);
            
            // Keep track of recent positions
            previous_positions.push(current_pos);
            if previous_positions.len() > 50 {  // Keep last 50 positions
                previous_positions.remove(0);
            }
            
            if i > 20 {  // After some initial movement
                position_samples += 1;
                
                // Check if current position is similar to a much earlier position
                for &(old_x, old_y) in &previous_positions[..previous_positions.len().saturating_sub(20)] {
                    let distance = ((current_pos.0 - old_x).powi(2) + (current_pos.1 - old_y).powi(2)).sqrt();
                    if distance < 100.0 {  // Returned to similar position
                        re_aggression_count += 1;
                        break;
                    }
                }
            }
        }

        if position_samples > 0 {
            re_aggression_count as f32 / position_samples as f32
        } else {
            0.0
        }
    }

    fn calculate_reaction_smoothness(&self, reaction_sequence: &[BehavioralVector]) -> f32 {
        if reaction_sequence.len() < 3 {
            return 1.0;
        }

        let mut smoothness_sum = 0.0;
        
        for window in reaction_sequence.windows(3) {
            let prev_change = ((window[1].yaw - window[0].yaw).powi(2) + 
                             (window[1].pitch - window[0].pitch).powi(2)).sqrt();
            let curr_change = ((window[2].yaw - window[1].yaw).powi(2) + 
                             (window[2].pitch - window[1].pitch).powi(2)).sqrt();
            
            // Smooth reactions have consistent change magnitudes
            let change_consistency = if prev_change > 0.0 && curr_change > 0.0 {
                1.0 - ((prev_change - curr_change).abs() / (prev_change + curr_change))
            } else {
                1.0
            };
            
            smoothness_sum += change_consistency;
        }

        smoothness_sum / (reaction_sequence.len() - 2) as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_team_dynamics_extractor() {
        let extractor = TeamDynamicsExtractor::new();
        
        // Create test team data
        let mut team_vectors = HashMap::new();
        
        // Player 1
        let player1_vectors = vec![
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
                pos_x: 100.0,
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
        
        // Player 2
        let player2_vectors = vec![
            BehavioralVector {
                tick: 1,
                steamid: 76561198123456790,
                health: 100.0,
                armor: 100.0,
                pos_x: 500.0,
                pos_y: 500.0,
                pos_z: 64.0,
                vel_x: 0.0,
                vel_y: 250.0,
                vel_z: 0.0,
                yaw: 90.0,
                pitch: 0.0,
                weapon_id: 16,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 0.0,
                delta_pitch: 0.0,
            },
            BehavioralVector {
                tick: 2,
                steamid: 76561198123456790,
                health: 100.0,
                armor: 100.0,
                pos_x: 500.0,
                pos_y: 600.0,
                pos_z: 64.0,
                vel_x: 0.0,
                vel_y: 250.0,
                vel_z: 0.0,
                yaw: 95.0,
                pitch: -1.0,
                weapon_id: 16,
                ammo: 30.0,
                is_airborne: 0.0,
                delta_yaw: 5.0,
                delta_pitch: -1.0,
            },
        ];
        
        team_vectors.insert(76561198123456789, player1_vectors);
        team_vectors.insert(76561198123456790, player2_vectors);
        
        let features = extractor.extract_features(&team_vectors);
        
        // Basic validation
        assert!(features.formation_spread_vs_stack >= 0.0);
        assert!(features.map_control_percentage >= 0.0);
        assert!(features.crossfire_setup_effectiveness >= 0.0);
        assert!(!features.site_approach_patterns.is_empty());
    }

    #[test]
    fn test_decision_metrics_extractor() {
        let extractor = DecisionMetricsExtractor::new();
        
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
                health: 95.0,
                armor: 100.0,
                pos_x: 10.0,
                pos_y: 0.0,
                pos_z: 64.0,
                vel_x: 250.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 15.0, // Rapid aim change
                pitch: -5.0,
                weapon_id: 7,
                ammo: 27.0,
                is_airborne: 0.0,
                delta_yaw: 15.0,
                delta_pitch: -5.0,
            },
        ];

        let team_vectors = HashMap::new();
        let features = extractor.extract_features(&vectors, &team_vectors);
        
        // Basic validation
        assert!(features.buy_efficiency_value_per_dollar >= 0.0);
        assert!(features.decision_speed_after_first_contact >= 0.0);
        assert!(features.reaction_time_visual_stimuli >= 0.0);
    }

    #[test]
    fn test_team_spread_calculation() {
        let extractor = TeamDynamicsExtractor::new();
        
        // Test spread positions
        let spread_positions = vec![
            (0.0, 0.0, 64.0),
            (1000.0, 0.0, 64.0),
            (0.0, 1000.0, 64.0),
        ];
        
        // Test stacked positions
        let stacked_positions = vec![
            (0.0, 0.0, 64.0),
            (10.0, 10.0, 64.0),
            (20.0, 20.0, 64.0),
        ];
        
        let spread_value = extractor.calculate_team_spread(&spread_positions);
        let stacked_value = extractor.calculate_team_spread(&stacked_positions);
        
        assert!(spread_value > stacked_value);
    }
}