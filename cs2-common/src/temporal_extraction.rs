use crate::feature_extraction::TemporalContextFeatures;
use crate::BehavioralVector;
use std::collections::HashMap;

/// Temporal Context Extractor - Analyzes round phases, map context, and opponent adaptation
pub struct TemporalContextExtractor {
    pub round_length_ticks: u32,         // Typical round length in ticks
    pub early_round_threshold: u32,      // Ticks defining early round
    pub late_round_threshold: u32,       // Ticks defining late round
    pub clutch_players_threshold: usize, // Max players alive to be considered clutch
}

impl Default for TemporalContextExtractor {
    fn default() -> Self {
        Self {
            round_length_ticks: 7000,    // ~110 seconds at 64 tick
            early_round_threshold: 1000, // ~15 seconds
            late_round_threshold: 5000,  // ~78 seconds
            clutch_players_threshold: 2,
        }
    }
}

impl TemporalContextExtractor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Extract temporal and contextual features from behavioral vectors
    pub fn extract_features(
        &self,
        vectors: &[BehavioralVector],
        team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
        map_name: Option<&str>,
    ) -> TemporalContextFeatures {
        let mut features = TemporalContextFeatures {
            early_round_tendencies: HashMap::new(),
            mid_round_adaptations: HashMap::new(),
            late_round_decision_patterns: HashMap::new(),
            clutch_performance_metrics: 0.0,
            map_specific_tendencies: HashMap::new(),
            position_preference_by_map: HashMap::new(),
            success_rates_by_area: HashMap::new(),
            route_preference_patterns: HashMap::new(),
            counter_strategy_effectiveness: 0.0,
            adaptation_to_opponent_patterns: 0.0,
            anti_strategy_timing: 0.0,
            information_denial_effectiveness: 0.0,
        };

        if vectors.is_empty() {
            return features;
        }

        // Extract round phase context
        self.extract_round_phase_context(&mut features, vectors);

        // Extract map context
        self.extract_map_context(&mut features, vectors, map_name);

        // Extract opponent adaptation metrics
        self.extract_opponent_adaptation(&mut features, vectors, team_vectors);

        features
    }

    fn extract_round_phase_context(
        &self,
        features: &mut TemporalContextFeatures,
        vectors: &[BehavioralVector],
    ) {
        // Segment vectors by round phases
        let (early_vectors, mid_vectors, late_vectors) = self.segment_by_round_phase(vectors);

        // Analyze early round tendencies
        features.early_round_tendencies = self.analyze_phase_tendencies(&early_vectors, "early");

        // Analyze mid round adaptations
        features.mid_round_adaptations = self.analyze_phase_adaptations(&mid_vectors, "mid");

        // Analyze late round decision patterns
        features.late_round_decision_patterns = self.analyze_phase_decisions(&late_vectors, "late");

        // Analyze clutch performance
        features.clutch_performance_metrics = self.analyze_clutch_performance(vectors);
    }

    fn extract_map_context(
        &self,
        features: &mut TemporalContextFeatures,
        vectors: &[BehavioralVector],
        map_name: Option<&str>,
    ) {
        let map = map_name.unwrap_or("unknown");

        // Analyze map-specific positioning tendencies
        let positioning_tendencies = self.analyze_map_positioning(vectors, map);
        features.map_specific_tendencies = positioning_tendencies;

        // Analyze position preferences by map areas
        let position_preferences = self.analyze_position_preferences(vectors, map);
        features
            .position_preference_by_map
            .insert(map.to_string(), position_preferences);

        // Analyze success rates by map areas
        features.success_rates_by_area = self.analyze_area_success_rates(vectors, map);

        // Analyze route preferences
        features.route_preference_patterns = self.analyze_route_preferences(vectors, map);
    }

    fn extract_opponent_adaptation(
        &self,
        features: &mut TemporalContextFeatures,
        vectors: &[BehavioralVector],
        team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) {
        // Analyze counter-strategy effectiveness
        features.counter_strategy_effectiveness =
            self.analyze_counter_strategy_effectiveness(vectors, team_vectors);

        // Analyze adaptation to opponent patterns
        features.adaptation_to_opponent_patterns =
            self.analyze_pattern_adaptation(vectors, team_vectors);

        // Analyze anti-strategy timing
        features.anti_strategy_timing = self.analyze_anti_strategy_timing(vectors, team_vectors);

        // Analyze information denial effectiveness
        features.information_denial_effectiveness =
            self.analyze_information_denial(vectors, team_vectors);
    }

    fn segment_by_round_phase<'a>(
        &self,
        vectors: &'a [BehavioralVector],
    ) -> (
        Vec<&'a BehavioralVector>,
        Vec<&'a BehavioralVector>,
        Vec<&'a BehavioralVector>,
    ) {
        if vectors.is_empty() {
            return (Vec::new(), Vec::new(), Vec::new());
        }

        let min_tick = vectors.iter().map(|v| v.tick).min().unwrap_or(0);

        let early: Vec<&BehavioralVector> = vectors
            .iter()
            .filter(|v| v.tick - min_tick <= self.early_round_threshold)
            .collect();

        let late: Vec<&BehavioralVector> = vectors
            .iter()
            .filter(|v| v.tick - min_tick >= self.late_round_threshold)
            .collect();

        let mid: Vec<&BehavioralVector> = vectors
            .iter()
            .filter(|v| {
                let relative_tick = v.tick - min_tick;
                relative_tick > self.early_round_threshold
                    && relative_tick < self.late_round_threshold
            })
            .collect();

        (early, mid, late)
    }

    fn analyze_phase_tendencies(
        &self,
        vectors: &[&BehavioralVector],
        phase: &str,
    ) -> HashMap<String, f32> {
        let mut tendencies = HashMap::new();

        if vectors.is_empty() {
            return tendencies;
        }

        // Analyze movement patterns in this phase
        let avg_speed = self.calculate_average_speed(vectors);
        let avg_position_stability = self.calculate_position_stability(vectors);
        let weapon_usage = self.analyze_weapon_usage_patterns(vectors);
        let aim_intensity = self.calculate_aim_intensity(vectors);

        tendencies.insert(format!("{phase}_avg_speed"), avg_speed);
        tendencies.insert(
            format!("{phase}_position_stability"),
            avg_position_stability,
        );
        tendencies.insert(format!("{phase}_aim_intensity"), aim_intensity);

        // Add weapon-specific tendencies
        for (weapon, usage) in weapon_usage {
            tendencies.insert(format!("{phase}_{weapon}_usage"), usage);
        }

        tendencies
    }

    fn analyze_phase_adaptations(
        &self,
        vectors: &[&BehavioralVector],
        phase: &str,
    ) -> HashMap<String, f32> {
        let mut adaptations = HashMap::new();

        if vectors.len() < 10 {
            return adaptations;
        }

        // Analyze adaptation frequency through position changes
        let adaptation_frequency = self.calculate_adaptation_frequency(vectors);
        adaptations.insert(
            format!("{phase}_adaptation_frequency"),
            adaptation_frequency,
        );

        // Analyze strategy persistence vs change
        let strategy_persistence = self.calculate_strategy_persistence(vectors);
        adaptations.insert(
            format!("{phase}_strategy_persistence"),
            strategy_persistence,
        );

        // Analyze decision reversal patterns
        let decision_reversals = self.calculate_decision_reversals(vectors);
        adaptations.insert(format!("{phase}_decision_reversals"), decision_reversals);

        adaptations
    }

    fn analyze_phase_decisions(
        &self,
        vectors: &[&BehavioralVector],
        phase: &str,
    ) -> HashMap<String, f32> {
        let mut decisions = HashMap::new();

        if vectors.is_empty() {
            return decisions;
        }

        // Analyze decision urgency (rapid changes)
        let decision_urgency = self.calculate_decision_urgency(vectors);
        decisions.insert(format!("{phase}_decision_urgency"), decision_urgency);

        // Analyze positioning conservatism
        let conservatism = self.calculate_positioning_conservatism(vectors);
        decisions.insert(format!("{phase}_conservatism"), conservatism);

        // Analyze risk-taking patterns
        let risk_taking = self.calculate_risk_taking_patterns(vectors);
        decisions.insert(format!("{phase}_risk_taking"), risk_taking);

        decisions
    }

    fn analyze_clutch_performance(&self, vectors: &[BehavioralVector]) -> f32 {
        // Simplified clutch detection and performance analysis
        // In real implementation, would need team state and elimination events

        let mut clutch_situations = 0;
        let mut successful_clutch_behaviors = 0;

        for window in vectors.windows(10) {
            // Detect potential clutch situations through isolated positioning
            let isolation_score = self.calculate_isolation_score(window);

            if isolation_score > 0.7 {
                // High isolation suggests potential clutch
                clutch_situations += 1;

                // Analyze clutch behavior quality
                let behavior_quality = self.analyze_clutch_behavior_quality(window);
                if behavior_quality > 0.6 {
                    successful_clutch_behaviors += 1;
                }
            }
        }

        if clutch_situations > 0 {
            successful_clutch_behaviors as f32 / clutch_situations as f32
        } else {
            0.0
        }
    }

    fn analyze_map_positioning(
        &self,
        vectors: &[BehavioralVector],
        map_name: &str,
    ) -> HashMap<String, f32> {
        let mut tendencies = HashMap::new();

        // Analyze positioning patterns specific to map areas
        let area_preferences = self.classify_map_areas(vectors, map_name);

        for (area, preference) in area_preferences {
            tendencies.insert(format!("{area}_preference"), preference);
        }

        // Analyze verticality usage (height preferences)
        let height_variance = self.calculate_height_variance(vectors);
        tendencies.insert("height_variation".to_string(), height_variance);

        // Analyze corner vs open space preferences
        let corner_preference = self.calculate_corner_preference(vectors);
        tendencies.insert("corner_preference".to_string(), corner_preference);

        tendencies
    }

    fn analyze_position_preferences(
        &self,
        vectors: &[BehavioralVector],
        map_name: &str,
    ) -> HashMap<String, f32> {
        let mut preferences = HashMap::new();

        // Define map-specific areas (simplified for demonstration)
        let areas = match map_name {
            "de_dust2" => vec!["long_a", "cat", "mid", "tunnels", "b_site", "a_site"],
            "de_mirage" => vec![
                "ramp",
                "apps",
                "mid",
                "jungle",
                "connector",
                "a_site",
                "b_site",
            ],
            "de_inferno" => vec![
                "apps", "arch", "mid", "alt_mid", "banana", "a_site", "b_site",
            ],
            _ => vec!["area_1", "area_2", "area_3", "area_4", "area_5"],
        };

        let total_time = vectors.len() as f32;

        for area in areas {
            let time_in_area = self.calculate_time_in_area(vectors, area, map_name);
            preferences.insert(area.to_string(), time_in_area / total_time);
        }

        preferences
    }

    fn analyze_area_success_rates(
        &self,
        vectors: &[BehavioralVector],
        map_name: &str,
    ) -> HashMap<String, f32> {
        let mut success_rates = HashMap::new();

        // Simplified success rate calculation based on health maintenance and positioning
        let areas = self.classify_map_areas(vectors, map_name);

        for (area, _) in areas {
            let area_vectors: Vec<&BehavioralVector> = vectors
                .iter()
                .filter(|v| self.position_in_area(v, &area, map_name))
                .collect();

            if !area_vectors.is_empty() {
                let avg_health =
                    area_vectors.iter().map(|v| v.health).sum::<f32>() / area_vectors.len() as f32;

                // Normalize health to success rate (simplified)
                success_rates.insert(area.clone(), avg_health / 100.0);
            }
        }

        success_rates
    }

    fn analyze_route_preferences(
        &self,
        vectors: &[BehavioralVector],
        map_name: &str,
    ) -> HashMap<String, f32> {
        let mut route_preferences = HashMap::new();

        // Analyze common movement patterns as routes
        let routes = self.detect_common_routes(vectors, map_name);
        let total_movements = routes.values().sum::<i32>() as f32;

        if total_movements > 0.0 {
            for (route, count) in routes {
                route_preferences.insert(route, count as f32 / total_movements);
            }
        }

        route_preferences
    }

    fn analyze_counter_strategy_effectiveness(
        &self,
        vectors: &[BehavioralVector],
        team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) -> f32 {
        // Analyze adaptation to opponent patterns
        // Simplified by looking at position changes relative to team movements

        let mut counter_moves = 0;
        let mut total_opportunities = 0;

        for window in vectors.windows(20) {
            // Detect opponent pattern (simplified)
            let opponent_pattern = self.detect_opponent_pattern_in_window(window, team_vectors);

            if opponent_pattern.is_some() {
                total_opportunities += 1;

                // Check if player adapted position in response
                let adaptation = self.detect_counter_adaptation(window);
                if adaptation > 0.5 {
                    counter_moves += 1;
                }
            }
        }

        if total_opportunities > 0 {
            counter_moves as f32 / total_opportunities as f32
        } else {
            0.0
        }
    }

    fn analyze_pattern_adaptation(
        &self,
        vectors: &[BehavioralVector],
        _team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) -> f32 {
        // Analyze how well player adapts to changing opponent patterns

        let mut adaptation_scores = Vec::new();

        // Split into segments to detect pattern changes
        let segment_size = vectors.len() / 4;
        if segment_size < 10 {
            return 0.0;
        }

        for i in 0..3 {
            let segment_start = i * segment_size;
            let segment_end = (i + 1) * segment_size;
            let next_segment_end = ((i + 2) * segment_size).min(vectors.len());

            if next_segment_end <= segment_end {
                continue;
            }

            let current_segment = &vectors[segment_start..segment_end];
            let next_segment = &vectors[segment_end..next_segment_end];

            // Measure behavioral change between segments
            let adaptation_score =
                self.measure_behavioral_adaptation(current_segment, next_segment);
            adaptation_scores.push(adaptation_score);
        }

        if !adaptation_scores.is_empty() {
            adaptation_scores.iter().sum::<f32>() / adaptation_scores.len() as f32
        } else {
            0.0
        }
    }

    fn analyze_anti_strategy_timing(
        &self,
        vectors: &[BehavioralVector],
        _team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) -> f32 {
        // Analyze timing of anti-strategy moves
        // Simplified by looking at unexpected position changes

        let mut anti_strategy_moves = 0;
        let mut total_strategic_moments = 0;

        for window in vectors.windows(30) {
            // Detect strategic moments (periods of consistent behavior)
            let consistency = self.calculate_behavior_consistency(window);

            if consistency > 0.7 {
                // High consistency suggests strategic behavior
                total_strategic_moments += 1;

                // Check for sudden change (anti-strategy)
                if window.len() > 20 {
                    let early_behavior = &window[0..10];
                    let late_behavior = &window[20..];

                    let behavior_change =
                        self.measure_behavior_change(early_behavior, late_behavior);
                    if behavior_change > 0.6 {
                        // Significant change
                        anti_strategy_moves += 1;
                    }
                }
            }
        }

        if total_strategic_moments > 0 {
            anti_strategy_moves as f32 / total_strategic_moments as f32
        } else {
            0.0
        }
    }

    fn analyze_information_denial(
        &self,
        vectors: &[BehavioralVector],
        _team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) -> f32 {
        // Analyze effectiveness of hiding information from opponents
        // Simplified by analyzing unpredictability of movements

        let mut unpredictability_scores = Vec::new();

        for window in vectors.windows(15) {
            let unpredictability = self.calculate_movement_unpredictability(window);
            unpredictability_scores.push(unpredictability);
        }

        if !unpredictability_scores.is_empty() {
            unpredictability_scores.iter().sum::<f32>() / unpredictability_scores.len() as f32
        } else {
            0.0
        }
    }

    // Helper methods

    fn calculate_average_speed(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        let total_speed: f32 = vectors
            .iter()
            .map(|v| (v.vel_x.powi(2) + v.vel_y.powi(2) + v.vel_z.powi(2)).sqrt())
            .sum();

        total_speed / vectors.len() as f32
    }

    fn calculate_position_stability(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.len() < 2 {
            return 1.0;
        }

        let mut total_movement = 0.0;

        for window in vectors.windows(2) {
            let dx = window[1].pos_x - window[0].pos_x;
            let dy = window[1].pos_y - window[0].pos_y;
            let movement = (dx.powi(2) + dy.powi(2)).sqrt();
            total_movement += movement;
        }

        let avg_movement = total_movement / (vectors.len() - 1) as f32;

        // Convert to stability score (inverse of movement, normalized)
        1.0 - (avg_movement / 500.0).min(1.0)
    }

    fn analyze_weapon_usage_patterns(&self, vectors: &[&BehavioralVector]) -> HashMap<String, f32> {
        let mut weapon_usage = HashMap::new();
        let mut weapon_counts: HashMap<u16, usize> = HashMap::new();

        for vector in vectors {
            *weapon_counts.entry(vector.weapon_id).or_insert(0) += 1;
        }

        let total_count = vectors.len() as f32;

        for (weapon_id, count) in weapon_counts {
            let weapon_name = self.weapon_id_to_name(weapon_id);
            weapon_usage.insert(weapon_name, count as f32 / total_count);
        }

        weapon_usage
    }

    fn calculate_aim_intensity(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.len() < 2 {
            return 0.0;
        }

        let mut total_aim_change = 0.0;

        for window in vectors.windows(2) {
            let yaw_change = (window[1].yaw - window[0].yaw).abs();
            let pitch_change = (window[1].pitch - window[0].pitch).abs();
            total_aim_change += (yaw_change.powi(2) + pitch_change.powi(2)).sqrt();
        }

        total_aim_change / (vectors.len() - 1) as f32
    }

    fn calculate_adaptation_frequency(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.len() < 10 {
            return 0.0;
        }

        let mut adaptations = 0;
        let segment_size = vectors.len() / 5;

        for i in 0..4 {
            let segment1_start = i * segment_size;
            let segment1_end = (i + 1) * segment_size;
            let segment2_start = segment1_end;
            let segment2_end = ((i + 2) * segment_size).min(vectors.len());

            if segment2_end <= segment2_start || segment1_end <= segment1_start {
                continue;
            }

            let segment1 = &vectors[segment1_start..segment1_end];
            let segment2 = &vectors[segment2_start..segment2_end];

            let behavior_change = self.measure_segment_behavior_change(segment1, segment2);
            if behavior_change > 0.5 {
                adaptations += 1;
            }
        }

        adaptations as f32 / 4.0
    }

    fn calculate_strategy_persistence(&self, vectors: &[&BehavioralVector]) -> f32 {
        // Higher values indicate more persistent strategies
        1.0 - self.calculate_adaptation_frequency(vectors)
    }

    fn calculate_decision_reversals(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.len() < 6 {
            return 0.0;
        }

        let mut reversals = 0;
        let mut total_decisions = 0;

        for window in vectors.windows(6) {
            // Look for A->B->A patterns in positioning
            let pos1 = (window[0].pos_x, window[0].pos_y);
            let pos2 = (window[2].pos_x, window[2].pos_y);
            let pos3 = (window[4].pos_x, window[4].pos_y);

            let dist_1_2 = ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2)).sqrt();
            let dist_1_3 = ((pos3.0 - pos1.0).powi(2) + (pos3.1 - pos1.1).powi(2)).sqrt();

            if dist_1_2 > 100.0 {
                // Significant move
                total_decisions += 1;

                if dist_1_3 < 50.0 {
                    // Returned to original position
                    reversals += 1;
                }
            }
        }

        if total_decisions > 0 {
            reversals as f32 / total_decisions as f32
        } else {
            0.0
        }
    }

    fn calculate_decision_urgency(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.len() < 3 {
            return 0.0;
        }

        let mut urgent_decisions = 0;
        let mut total_decisions = 0;

        for window in vectors.windows(3) {
            let pos_change_1 = ((window[1].pos_x - window[0].pos_x).powi(2)
                + (window[1].pos_y - window[0].pos_y).powi(2))
            .sqrt();
            let pos_change_2 = ((window[2].pos_x - window[1].pos_x).powi(2)
                + (window[2].pos_y - window[1].pos_y).powi(2))
            .sqrt();

            if pos_change_1 > 50.0 || pos_change_2 > 50.0 {
                total_decisions += 1;

                // Rapid successive changes indicate urgency
                if pos_change_1 > 100.0 && pos_change_2 > 100.0 {
                    urgent_decisions += 1;
                }
            }
        }

        if total_decisions > 0 {
            urgent_decisions as f32 / total_decisions as f32
        } else {
            0.0
        }
    }

    fn calculate_positioning_conservatism(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        // Calculate average distance from spawn/safe areas (simplified)
        let avg_distance_from_origin: f32 = vectors
            .iter()
            .map(|v| (v.pos_x.powi(2) + v.pos_y.powi(2)).sqrt())
            .sum::<f32>()
            / vectors.len() as f32;

        // Normalize conservatism (closer to origin = more conservative)
        1.0 - (avg_distance_from_origin / 2000.0).min(1.0)
    }

    fn calculate_risk_taking_patterns(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        // Risk-taking indicated by aggressive positioning and rapid movements
        let avg_speed = self.calculate_average_speed(vectors);
        let position_variance = self.calculate_position_variance(vectors);

        // Combine speed and position variance as risk indicators
        let speed_factor = (avg_speed / 300.0).min(1.0);
        let variance_factor = (position_variance / 1000.0).min(1.0);

        (speed_factor + variance_factor) / 2.0
    }

    fn calculate_isolation_score(&self, vectors: &[BehavioralVector]) -> f32 {
        // Simplified isolation calculation
        // In real implementation, would need teammate positions
        let position_variance = vectors
            .iter()
            .map(|v| (v.pos_x.powi(2) + v.pos_y.powi(2)).sqrt())
            .collect::<Vec<f32>>();

        if position_variance.is_empty() {
            return 0.0;
        }

        let mean_distance = position_variance.iter().sum::<f32>() / position_variance.len() as f32;

        // Higher distance from center suggests isolation
        (mean_distance / 1500.0).min(1.0)
    }

    fn analyze_clutch_behavior_quality(&self, vectors: &[BehavioralVector]) -> f32 {
        // Analyze quality of behavior in clutch situations
        let movement_efficiency = self.calculate_movement_efficiency(vectors);
        let aim_stability = self.calculate_aim_stability(vectors);
        let positioning_quality = self.calculate_positioning_quality(vectors);

        (movement_efficiency + aim_stability + positioning_quality) / 3.0
    }

    fn classify_map_areas(
        &self,
        vectors: &[BehavioralVector],
        map_name: &str,
    ) -> HashMap<String, f32> {
        let mut area_times = HashMap::new();
        let total_time = vectors.len() as f32;

        for vector in vectors {
            let area = self.position_to_area(vector, map_name);
            *area_times.entry(area).or_insert(0.0) += 1.0;
        }

        // Convert to preferences (time ratios)
        for (_, time) in area_times.iter_mut() {
            *time /= total_time;
        }

        area_times
    }

    fn calculate_height_variance(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        let heights: Vec<f32> = vectors.iter().map(|v| v.pos_z).collect();
        let mean_height = heights.iter().sum::<f32>() / heights.len() as f32;
        let variance = heights
            .iter()
            .map(|&h| (h - mean_height).powi(2))
            .sum::<f32>()
            / heights.len() as f32;

        variance.sqrt()
    }

    fn calculate_corner_preference(&self, _vectors: &[BehavioralVector]) -> f32 {
        // Simplified corner detection based on position clustering
        // In real implementation, would use actual map geometry
        0.6 // Placeholder value
    }

    fn calculate_time_in_area(
        &self,
        vectors: &[BehavioralVector],
        area: &str,
        map_name: &str,
    ) -> f32 {
        vectors
            .iter()
            .filter(|v| self.position_in_area(v, area, map_name))
            .count() as f32
    }

    fn position_in_area(&self, vector: &BehavioralVector, area: &str, _map_name: &str) -> bool {
        // Simplified area detection based on position ranges
        match area {
            "area_1" => vector.pos_x < 0.0 && vector.pos_y < 0.0,
            "area_2" => vector.pos_x >= 0.0 && vector.pos_y < 0.0,
            "area_3" => vector.pos_x < 0.0 && vector.pos_y >= 0.0,
            "area_4" => vector.pos_x >= 0.0 && vector.pos_y >= 0.0,
            _ => false,
        }
    }

    fn detect_common_routes(
        &self,
        vectors: &[BehavioralVector],
        _map_name: &str,
    ) -> HashMap<String, i32> {
        let mut routes = HashMap::new();

        // Simplified route detection through position sequences
        for window in vectors.windows(10) {
            let route_signature = self.calculate_route_signature(window);
            *routes.entry(route_signature).or_insert(0) += 1;
        }

        routes
    }

    fn calculate_route_signature(&self, vectors: &[BehavioralVector]) -> String {
        // Simplified route signature based on general movement direction
        if vectors.len() < 2 {
            return "static".to_string();
        }

        let start_pos = (vectors[0].pos_x, vectors[0].pos_y);
        let end_pos = (
            vectors[vectors.len() - 1].pos_x,
            vectors[vectors.len() - 1].pos_y,
        );

        let dx = end_pos.0 - start_pos.0;
        let dy = end_pos.1 - start_pos.1;

        match (dx > 0.0, dy > 0.0) {
            (true, true) => "northeast".to_string(),
            (true, false) => "southeast".to_string(),
            (false, true) => "northwest".to_string(),
            (false, false) => "southwest".to_string(),
        }
    }

    fn detect_opponent_pattern_in_window(
        &self,
        _vectors: &[BehavioralVector],
        _team_vectors: &HashMap<u64, Vec<BehavioralVector>>,
    ) -> Option<String> {
        // Simplified opponent pattern detection
        // In real implementation, would analyze teammate behaviors for opponent predictions
        Some("opponent_pattern".to_string())
    }

    fn detect_counter_adaptation(&self, vectors: &[BehavioralVector]) -> f32 {
        // Detect position changes that could be counter-adaptations
        if vectors.len() < 5 {
            return 0.0;
        }

        let early_pos = (vectors[0].pos_x, vectors[0].pos_y);
        let late_pos = (
            vectors[vectors.len() - 1].pos_x,
            vectors[vectors.len() - 1].pos_y,
        );

        let distance_moved =
            ((late_pos.0 - early_pos.0).powi(2) + (late_pos.1 - early_pos.1).powi(2)).sqrt();

        (distance_moved / 500.0).min(1.0)
    }

    fn measure_behavioral_adaptation(
        &self,
        segment1: &[BehavioralVector],
        segment2: &[BehavioralVector],
    ) -> f32 {
        if segment1.is_empty() || segment2.is_empty() {
            return 0.0;
        }

        // Compare average behaviors between segments
        let avg_speed_1 = self.calculate_segment_avg_speed(segment1);
        let avg_speed_2 = self.calculate_segment_avg_speed(segment2);

        let avg_pos_1 = self.calculate_segment_avg_position(segment1);
        let avg_pos_2 = self.calculate_segment_avg_position(segment2);

        let speed_change = (avg_speed_2 - avg_speed_1).abs() / avg_speed_1.max(1.0);
        let position_change =
            ((avg_pos_2.0 - avg_pos_1.0).powi(2) + (avg_pos_2.1 - avg_pos_1.1).powi(2)).sqrt();

        ((speed_change + position_change / 500.0) / 2.0).min(1.0)
    }

    fn calculate_behavior_consistency(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < 3 {
            return 1.0;
        }

        let mut consistency_scores = Vec::new();

        for window in vectors.windows(3) {
            let pos_var = self.calculate_position_variance_in_window(window);
            let speed_var = self.calculate_speed_variance_in_window(window);

            let consistency = 1.0 - ((pos_var / 100.0) + (speed_var / 50.0)).min(1.0);
            consistency_scores.push(consistency);
        }

        consistency_scores.iter().sum::<f32>() / consistency_scores.len() as f32
    }

    fn measure_behavior_change(
        &self,
        early: &[BehavioralVector],
        late: &[BehavioralVector],
    ) -> f32 {
        self.measure_behavioral_adaptation(early, late)
    }

    fn calculate_movement_unpredictability(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < 3 {
            return 0.0;
        }

        let mut direction_changes = 0;

        for window in vectors.windows(3) {
            let dir1 = self.calculate_movement_direction(&window[0], &window[1]);
            let dir2 = self.calculate_movement_direction(&window[1], &window[2]);

            let angle_change = (dir2 - dir1).abs();
            if angle_change > 45.0 {
                // Significant direction change
                direction_changes += 1;
            }
        }

        direction_changes as f32 / (vectors.len() - 2) as f32
    }

    // Additional helper methods

    fn weapon_id_to_name(&self, weapon_id: u16) -> String {
        match weapon_id {
            7 => "ak47".to_string(),
            16 => "m4a4".to_string(),
            60 => "m4a1s".to_string(),
            40 => "awp".to_string(),
            _ => format!("weapon_{weapon_id}"),
        }
    }

    fn measure_segment_behavior_change(
        &self,
        segment1: &[&BehavioralVector],
        segment2: &[&BehavioralVector],
    ) -> f32 {
        if segment1.is_empty() || segment2.is_empty() {
            return 0.0;
        }

        let speed1 = self.calculate_average_speed(segment1);
        let speed2 = self.calculate_average_speed(segment2);

        let pos1 = self.calculate_avg_position_from_refs(segment1);
        let pos2 = self.calculate_avg_position_from_refs(segment2);

        let speed_change = (speed2 - speed1).abs() / speed1.max(1.0);
        let pos_change = ((pos2.0 - pos1.0).powi(2) + (pos2.1 - pos1.1).powi(2)).sqrt();

        ((speed_change + pos_change / 500.0) / 2.0).min(1.0)
    }

    fn calculate_position_variance(&self, vectors: &[&BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        let mean_x = vectors.iter().map(|v| v.pos_x).sum::<f32>() / vectors.len() as f32;
        let mean_y = vectors.iter().map(|v| v.pos_y).sum::<f32>() / vectors.len() as f32;

        let variance = vectors
            .iter()
            .map(|v| ((v.pos_x - mean_x).powi(2) + (v.pos_y - mean_y).powi(2)))
            .sum::<f32>()
            / vectors.len() as f32;

        variance.sqrt()
    }

    fn calculate_movement_efficiency(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < 2 {
            return 1.0;
        }

        let mut total_distance = 0.0;
        for window in vectors.windows(2) {
            let actual_dist = ((window[1].pos_x - window[0].pos_x).powi(2)
                + (window[1].pos_y - window[0].pos_y).powi(2))
            .sqrt();
            total_distance += actual_dist;
        }

        // Calculate direct distance from start to end
        let start = &vectors[0];
        let end = &vectors[vectors.len() - 1];

        let optimal_distance =
            ((end.pos_x - start.pos_x).powi(2) + (end.pos_y - start.pos_y).powi(2)).sqrt();

        if total_distance > 0.0 {
            optimal_distance / total_distance
        } else {
            1.0
        }
    }

    fn calculate_aim_stability(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < 2 {
            return 1.0;
        }

        let mut total_aim_change = 0.0;

        for window in vectors.windows(2) {
            let yaw_change = (window[1].yaw - window[0].yaw).abs();
            let pitch_change = (window[1].pitch - window[0].pitch).abs();
            total_aim_change += (yaw_change.powi(2) + pitch_change.powi(2)).sqrt();
        }

        let avg_aim_change = total_aim_change / (vectors.len() - 1) as f32;

        // Stability is inverse of change (normalized)
        1.0 - (avg_aim_change / 90.0).min(1.0)
    }

    fn calculate_positioning_quality(&self, vectors: &[BehavioralVector]) -> f32 {
        // Simplified positioning quality based on health maintenance
        if vectors.is_empty() {
            return 1.0;
        }

        let avg_health = vectors.iter().map(|v| v.health).sum::<f32>() / vectors.len() as f32;
        avg_health / 100.0
    }

    fn position_to_area(&self, vector: &BehavioralVector, _map_name: &str) -> String {
        // Simplified area classification
        match (vector.pos_x > 0.0, vector.pos_y > 0.0) {
            (true, true) => "northeast".to_string(),
            (true, false) => "southeast".to_string(),
            (false, true) => "northwest".to_string(),
            (false, false) => "southwest".to_string(),
        }
    }

    fn calculate_segment_avg_speed(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        let total_speed: f32 = vectors
            .iter()
            .map(|v| (v.vel_x.powi(2) + v.vel_y.powi(2) + v.vel_z.powi(2)).sqrt())
            .sum();

        total_speed / vectors.len() as f32
    }

    fn calculate_segment_avg_position(&self, vectors: &[BehavioralVector]) -> (f32, f32) {
        if vectors.is_empty() {
            return (0.0, 0.0);
        }

        let sum_x = vectors.iter().map(|v| v.pos_x).sum::<f32>();
        let sum_y = vectors.iter().map(|v| v.pos_y).sum::<f32>();

        (sum_x / vectors.len() as f32, sum_y / vectors.len() as f32)
    }

    fn calculate_avg_position_from_refs(&self, vectors: &[&BehavioralVector]) -> (f32, f32) {
        if vectors.is_empty() {
            return (0.0, 0.0);
        }

        let sum_x = vectors.iter().map(|v| v.pos_x).sum::<f32>();
        let sum_y = vectors.iter().map(|v| v.pos_y).sum::<f32>();

        (sum_x / vectors.len() as f32, sum_y / vectors.len() as f32)
    }

    fn calculate_position_variance_in_window(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.len() < 2 {
            return 0.0;
        }

        let mean_x = vectors.iter().map(|v| v.pos_x).sum::<f32>() / vectors.len() as f32;
        let mean_y = vectors.iter().map(|v| v.pos_y).sum::<f32>() / vectors.len() as f32;

        let variance = vectors
            .iter()
            .map(|v| (v.pos_x - mean_x).powi(2) + (v.pos_y - mean_y).powi(2))
            .sum::<f32>()
            / vectors.len() as f32;

        variance.sqrt()
    }

    fn calculate_speed_variance_in_window(&self, vectors: &[BehavioralVector]) -> f32 {
        if vectors.is_empty() {
            return 0.0;
        }

        let speeds: Vec<f32> = vectors
            .iter()
            .map(|v| (v.vel_x.powi(2) + v.vel_y.powi(2) + v.vel_z.powi(2)).sqrt())
            .collect();

        let mean_speed = speeds.iter().sum::<f32>() / speeds.len() as f32;
        let variance = speeds
            .iter()
            .map(|&s| (s - mean_speed).powi(2))
            .sum::<f32>()
            / speeds.len() as f32;

        variance.sqrt()
    }

    fn calculate_movement_direction(&self, from: &BehavioralVector, to: &BehavioralVector) -> f32 {
        let dx = to.pos_x - from.pos_x;
        let dy = to.pos_y - from.pos_y;

        dy.atan2(dx).to_degrees()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_temporal_context_extractor() {
        let extractor = TemporalContextExtractor::new();

        let vectors = vec![
            BehavioralVector {
                tick: 100,
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
                tick: 3000,
                steamid: 76561198123456789,
                health: 90.0,
                armor: 100.0,
                pos_x: 500.0,
                pos_y: 500.0,
                pos_z: 64.0,
                vel_x: 100.0,
                vel_y: 100.0,
                vel_z: 0.0,
                yaw: 45.0,
                pitch: -10.0,
                weapon_id: 16,
                ammo: 25.0,
                is_airborne: 0.0,
                delta_yaw: 45.0,
                delta_pitch: -10.0,
            },
            BehavioralVector {
                tick: 6000,
                steamid: 76561198123456789,
                health: 75.0,
                armor: 80.0,
                pos_x: 1000.0,
                pos_y: 200.0,
                pos_z: 64.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 90.0,
                pitch: 0.0,
                weapon_id: 40,
                ammo: 10.0,
                is_airborne: 0.0,
                delta_yaw: 45.0,
                delta_pitch: 10.0,
            },
        ];

        let team_vectors = HashMap::new();
        let features = extractor.extract_features(&vectors, &team_vectors, Some("de_dust2"));

        // Basic validation
        assert!(!features.early_round_tendencies.is_empty());
        assert!(!features.map_specific_tendencies.is_empty());
        assert!(features.clutch_performance_metrics >= 0.0);
        assert!(features.counter_strategy_effectiveness >= 0.0);
    }

    #[test]
    fn test_round_phase_segmentation() {
        let extractor = TemporalContextExtractor::new();

        let vectors = vec![
            BehavioralVector {
                tick: 500, // Early round
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
                tick: 3000, // Mid round
                steamid: 76561198123456789,
                health: 90.0,
                armor: 100.0,
                pos_x: 500.0,
                pos_y: 500.0,
                pos_z: 64.0,
                vel_x: 100.0,
                vel_y: 100.0,
                vel_z: 0.0,
                yaw: 45.0,
                pitch: -10.0,
                weapon_id: 16,
                ammo: 25.0,
                is_airborne: 0.0,
                delta_yaw: 45.0,
                delta_pitch: -10.0,
            },
            BehavioralVector {
                tick: 6000, // Late round
                steamid: 76561198123456789,
                health: 75.0,
                armor: 80.0,
                pos_x: 1000.0,
                pos_y: 200.0,
                pos_z: 64.0,
                vel_x: 0.0,
                vel_y: 0.0,
                vel_z: 0.0,
                yaw: 90.0,
                pitch: 0.0,
                weapon_id: 40,
                ammo: 10.0,
                is_airborne: 0.0,
                delta_yaw: 45.0,
                delta_pitch: 10.0,
            },
        ];

        let (early, mid, late) = extractor.segment_by_round_phase(&vectors);

        assert_eq!(early.len(), 1);
        assert_eq!(mid.len(), 1);
        assert_eq!(late.len(), 1);
        assert_eq!(early[0].tick, 500);
        assert_eq!(mid[0].tick, 3000);
        assert_eq!(late[0].tick, 6000);
    }

    #[test]
    fn test_movement_unpredictability() {
        let extractor = TemporalContextExtractor::new();

        // Create predictable movement (straight line)
        let predictable_vectors: Vec<BehavioralVector> = (0..10)
            .map(|i| BehavioralVector {
                tick: i,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 100.0,
                pos_x: i as f32 * 100.0,
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

        // Create unpredictable movement (zigzag)
        let unpredictable_vectors: Vec<BehavioralVector> = (0..10)
            .map(|i| BehavioralVector {
                tick: i,
                steamid: 76561198123456789,
                health: 100.0,
                armor: 100.0,
                pos_x: if i % 2 == 0 { 0.0 } else { 100.0 },
                pos_y: i as f32 * 50.0,
                pos_z: 64.0,
                vel_x: if i % 2 == 0 { 250.0 } else { -250.0 },
                vel_y: 100.0,
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

        let predictable_score = extractor.calculate_movement_unpredictability(&predictable_vectors);
        let unpredictable_score =
            extractor.calculate_movement_unpredictability(&unpredictable_vectors);

        assert!(unpredictable_score > predictable_score);
    }
}
