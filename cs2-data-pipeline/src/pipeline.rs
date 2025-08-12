use anyhow::{anyhow, Result};
use chrono::Utc;
use futures::stream::{self, StreamExt};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::database::DatabaseManager;
use crate::models::{
    KeyMoment, KeyMomentType, Match, MomentBehavior, PlayerSnapshot, ProcessingStatus,
};

use cs2_demo_parser::first_pass::parser_settings::ParserInputs;
use cs2_demo_parser::parse_demo::{DemoOutput, Parser, ParsingMode};

#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub max_concurrent_jobs: usize,
    pub batch_size: usize,
    pub demo_directory: PathBuf,
    pub temp_directory: PathBuf,
    pub enable_ai_analysis: bool,
    pub chunk_size_ticks: u32,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            max_concurrent_jobs: 4,
            batch_size: 1000,
            demo_directory: PathBuf::from("./demos"),
            temp_directory: PathBuf::from("./temp"),
            enable_ai_analysis: true,
            chunk_size_ticks: 64 * 60,
        }
    }
}

pub struct DemoProcessor {
    db: Arc<DatabaseManager>,
    config: PipelineConfig,
    semaphore: Arc<Semaphore>,
}

impl DemoProcessor {
    pub fn new(db: DatabaseManager, config: PipelineConfig) -> Self {
        Self {
            db: Arc::new(db),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_jobs)),
            config,
        }
    }

    // NEW: public getters (used by tests and main.rs)
    pub fn db(&self) -> &DatabaseManager {
        Arc::as_ref(&self.db)
    }
    pub fn config(&self) -> &PipelineConfig {
        &self.config
    }

    pub async fn run(&self) -> Result<()> {
        info!("Pipeline init");
        self.db.postgres.initialize_schema().await?;
        self.db.timescale.initialize_schema().await?;
        self.db.vector.initialize_collections().await?;

        for demo in self.discover_demos().await? {
            if let Err(e) = self.register_demo(&demo).await {
                warn!("Register demo {} failed: {e}", demo.display());
            }
        }

        self.process_pending_matches().await?;
        info!("Pipeline done");
        Ok(())
    }

    pub async fn discover_demos(&self) -> Result<Vec<PathBuf>> {
        use walkdir::WalkDir;
        let mut out = Vec::new();
        for entry in WalkDir::new(&self.config.demo_directory) {
            let e = entry?;
            if e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("dem"))
                .unwrap_or(false)
            {
                out.push(e.path().to_path_buf());
            }
        }
        info!("Discovered {} demos", out.len());
        Ok(out)
    }

    pub async fn register_demo(&self, path: &Path) -> Result<Uuid> {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");
        let size = std::fs::metadata(path)?.len() as i64;
        let parts: Vec<&str> = stem.split('_').collect();
        let (tournament, team1, team2, map_name) = if parts.len() >= 5 {
            (
                Some(parts[0].to_string()),
                parts[1].to_string(),
                parts[3].to_string(),
                parts[4].to_string(),
            )
        } else {
            (None, "Team1".into(), "Team2".into(), "unknown".into())
        };
        let m = Match {
            id: Uuid::now_v7(),
            match_id: stem.to_string(),
            tournament,
            map_name,
            team1,
            team2,
            score_team1: 0,
            score_team2: 0,
            demo_file_path: path.to_string_lossy().to_string(),
            demo_file_size: size,
            tick_rate: 64,
            duration_seconds: 0,
            created_at: Utc::now(),
            processed_at: None,
            processing_status: ProcessingStatus::Pending,
        };
        self.db.postgres.insert_match(&m).await?;
        Ok(m.id)
    }

    pub async fn process_pending_matches(&self) -> Result<()> {
        let matches = self.db.postgres.get_unprocessed_matches().await?;
        stream::iter(matches)
            .map(|m| {
                let db = self.db.clone();
                let cfg = self.config.clone();
                let sem = self.semaphore.clone();
                async move {
                    let _permit = sem.acquire().await.unwrap();
                    Self::process_single_match(db, cfg, m).await
                }
            })
            .buffer_unordered(self.config.max_concurrent_jobs)
            .for_each(|res| async {
                if let Err(e) = res {
                    error!("match processing failed: {e:?}");
                }
            })
            .await;
        Ok(())
    }

    async fn process_single_match(
        db: Arc<DatabaseManager>,
        config: PipelineConfig,
        mut m: Match,
    ) -> Result<()> {
        info!("Processing {}", m.match_id);
        db.postgres
            .update_match_status(&m.match_id, ProcessingStatus::Processing)
            .await?;
        let res = Self::parse_and_persist(&db, &config, &mut m).await;
        match res {
            Ok(_) => {
                db.postgres
                    .update_match_status(&m.match_id, ProcessingStatus::Completed)
                    .await?;
                info!("Completed {}", m.match_id);
            }
            Err(e) => {
                error!("Error {}: {e:?}", m.match_id);
                db.postgres
                    .update_match_status(&m.match_id, ProcessingStatus::Failed)
                    .await?;
            }
        }
        Ok(())
    }

    fn build_parser_inputs(bytes: &[u8]) -> ParserInputs<'_> {
        static EMPTY_LOOKUP: OnceLock<Vec<(u8, u8)>> = OnceLock::new();
        let lut = EMPTY_LOOKUP.get_or_init(Vec::new);

        ParserInputs {
            real_name_to_og_name: ahash::AHashMap::new(),
            wanted_players: Vec::new(),
            wanted_player_props: vec![
                "X",
                "Y",
                "Z",
                "health",
                "armor_value",
                "velocity[0]",
                "velocity[1]",
                "velocity[2]",
                "m_angEyeAngles[0]",
                "m_angEyeAngles[1]",
                "m_hActiveWeapon",
                "m_iClip1",
                "m_lifeState",
                "m_hGroundEntity",
                "m_bIsScoped",
                "m_bIsWalking",
                "m_flFlashDuration",
                "m_iAccount",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            wanted_other_props: vec![],
            wanted_prop_states: ahash::AHashMap::new(),
            wanted_ticks: vec![],
            wanted_events: vec![
                "round_start",
                "round_end",
                "player_death",
                "weapon_fire",
                "player_hurt",
                "bomb_planted",
                "bomb_defused",
                "bomb_exploded",
            ]
            .into_iter()
            .map(|s| s.to_string())
            .collect(),
            parse_ents: true,
            parse_projectiles: true,
            parse_grenades: true,
            only_header: false,
            only_convars: false,
            huffman_lookup_table: lut, // &Vec<(u8,u8)>
            order_by_steamid: true,
            list_props: false,
            fallback_bytes: Some(bytes.to_vec()),
        }
    }

    async fn parse_and_persist(
        db: &Arc<DatabaseManager>,
        config: &PipelineConfig,
        m: &mut Match,
    ) -> Result<()> {
        let path = Path::new(&m.demo_file_path);
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| anyhow!("read demo {}: {e}", path.display()))?;
        let inputs = Self::build_parser_inputs(&bytes);
        let mut parser = Parser::new(inputs, ParsingMode::ForceMultiThreaded);
        let out = parser
            .parse_demo(&bytes)
            .map_err(|e| anyhow!("parse failure: {e:?}"))?;

        if out.header.is_some() {
            m.tick_rate = 64;
            m.duration_seconds = (out.game_events.len() as f32 / 64.0) as i32;
        }

        Self::ingest_player_snapshots(db, config, m, &out).await?;

        let moments = Self::detect_key_moments(m, &out).await?;
        if !moments.is_empty() {
            Self::persist_key_moments_and_behaviors(db, m, &out, &moments).await?;
            info!("Persisted {} key moments for {}", moments.len(), m.match_id);
        } else {
            info!("No key moments for {}", m.match_id);
        }

        Ok(())
    }

    async fn ingest_player_snapshots(
        db: &Arc<DatabaseManager>,
        config: &PipelineConfig,
        match_data: &Match,
        _out: &DemoOutput,
    ) -> Result<()> {
        // TODO: Implement actual extraction from out.df -> PlayerSnapshot
        let batch: Vec<PlayerSnapshot> = Vec::new();
        if !batch.is_empty() {
            db.timescale.insert_snapshots_batch(&batch).await?;
            if config.enable_ai_analysis {
                let embeddings: Vec<Vec<f32>> = batch
                    .iter()
                    .map(|s| {
                        vec![
                            s.pos_x,
                            s.pos_y,
                            s.pos_z,
                            s.vel_x,
                            s.vel_y,
                            s.vel_z,
                            s.yaw,
                            s.pitch,
                            s.flash_duration,
                            s.health,
                            s.armor,
                            s.ammo_clip as f32,
                            s.ammo_reserve as f32,
                            s.money as f32,
                            s.is_scoped as i32 as f32,
                            s.is_walking as i32 as f32,
                        ]
                    })
                    .collect();
                db.vector
                    .upsert_snapshot_embeddings(match_data.id, 1, &batch, &embeddings)
                    .await
                    .ok();
            }
        }
        Ok(())
    }

    // v4 moment detection (uses serde_json view of GameEvent)
    async fn detect_key_moments(match_data: &Match, out: &DemoOutput) -> Result<Vec<KeyMoment>> {
        const TICK_RATE: u32 = 64;
        const TRADE_WINDOW_TICKS: u32 = 5 * TICK_RATE;
        const EXECUTE_CLUSTER_WINDOW: u32 = 10 * TICK_RATE;
        const MOMENT_PAD_BEFORE: u32 = 2 * TICK_RATE;
        const MOMENT_PAD_AFTER: u32 = 5 * TICK_RATE;

        let mut moments: Vec<KeyMoment> = Vec::new();

        // Round trackers
        #[allow(unused_variables)]
        let mut round_number: i32 = 0;
        let mut round_start_tick: u32 = 0;
        let mut round_first_blood_done = false;
        let mut kills_this_round: HashMap<i64, u32> = HashMap::new();

        let mut alive_t: i32 = 5;
        let mut alive_ct: i32 = 5;

        let mut last_deaths_window: VecDeque<(u32, i64, i64)> = VecDeque::new();

        let mut current_plant_tick: Option<u32> = None;
        let mut postplant_ct_kills: Vec<(u32, i64, i64)> = Vec::new();
        let mut preplant_t_kills: Vec<(u32, i64, i64)> = Vec::new();

        let mut min_alive_t = 5;
        let mut min_alive_ct = 5;

        for ev in &out.game_events {
            let tick = ev.tick as u32;
            match ev.name.as_str() {
                "round_start" => {
                    round_number += 1;
                    round_start_tick = tick;
                    round_first_blood_done = false;
                    kills_this_round.clear();
                    last_deaths_window.clear();
                    current_plant_tick = None;
                    postplant_ct_kills.clear();
                    preplant_t_kills.clear();
                    alive_t = 5;
                    alive_ct = 5;
                    min_alive_t = 5;
                    min_alive_ct = 5;
                }
                "player_death" => {
                    let obj = serde_json::to_value(ev).unwrap_or(JsonValue::Null);

                    let victim =
                        get_i64(&obj, &["userid", "victim", "victim_steamid"]).unwrap_or(-1);
                    let killer =
                        get_i64(&obj, &["attacker", "killer", "attacker_steamid"]).unwrap_or(-1);
                    let headshot = get_bool(&obj, &["headshot"]).unwrap_or(false);
                    let weapon = get_str(&obj, &["weapon"]);
                    let victim_team = get_str(&obj, &["userteam", "victimteam", "victim_team"]);
                    let killer_team =
                        get_str(&obj, &["attackerteam", "killerteam", "attacker_team"]);

                    match victim_team.as_deref() {
                        Some("T") | Some("t") => alive_t = (alive_t - 1).max(0),
                        Some("CT") | Some("ct") => alive_ct = (alive_ct - 1).max(0),
                        _ => {}
                    }
                    min_alive_t = min_alive_t.min(alive_t);
                    min_alive_ct = min_alive_ct.min(alive_ct);

                    // Opening duel
                    if !round_first_blood_done && killer != -1 && victim != -1 {
                        round_first_blood_done = true;
                        let start_tick = tick.saturating_sub(MOMENT_PAD_BEFORE);
                        let end_tick = tick + MOMENT_PAD_AFTER;
                        let diff_before = (alive_t - alive_ct).abs();
                        moments.push(KeyMoment {
                            id: Uuid::now_v7(),
                            match_id: match_data.id,
                            moment_type: KeyMomentType::ImportantDuel,
                            start_tick,
                            end_tick,
                            players_involved: vec![killer, victim],
                            outcome: format!(
                                "Opening duel: {} killed {}{}{}",
                                killer,
                                victim,
                                if headshot { " (HS)" } else { "" },
                                weapon
                                    .as_ref()
                                    .map(|w| format!(" with {}", w))
                                    .unwrap_or_default()
                            ),
                            importance_score: 0.6 + (diff_before as f32 * 0.05),
                            created_at: Utc::now(),
                        });
                    }

                    // Pre / post plant clusters
                    if current_plant_tick.is_none() {
                        if killer_team
                            .as_deref()
                            .map(|t| t.eq_ignore_ascii_case("T"))
                            .unwrap_or(false)
                            && killer != -1
                            && victim != -1
                        {
                            preplant_t_kills.push((tick, killer, victim));
                        }
                    } else if killer_team
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case("CT"))
                        .unwrap_or(false)
                        && killer != -1
                        && victim != -1
                    {
                        postplant_ct_kills.push((tick, killer, victim));
                    }

                    // Streaks / Ace
                    if killer != -1 {
                        let cnt = kills_this_round.entry(killer).or_insert(0);
                        *cnt += 1;
                        if *cnt == 5 {
                            moments.push(KeyMoment {
                                id: Uuid::now_v7(),
                                match_id: match_data.id,
                                moment_type: KeyMomentType::Ace,
                                start_tick: round_start_tick,
                                end_tick: tick + MOMENT_PAD_AFTER,
                                players_involved: vec![killer],
                                outcome: format!("Ace by {}", killer),
                                importance_score: 0.9,
                                created_at: Utc::now(),
                            });
                        } else if *cnt >= 2 {
                            moments.push(KeyMoment {
                                id: Uuid::now_v7(),
                                match_id: match_data.id,
                                moment_type: KeyMomentType::ImportantDuel,
                                start_tick: tick.saturating_sub(MOMENT_PAD_BEFORE),
                                end_tick: tick + MOMENT_PAD_AFTER,
                                players_involved: vec![killer, victim],
                                outcome: format!(
                                    "{}-kill streak updated ({} -> {})",
                                    *cnt, killer, victim
                                ),
                                importance_score: 0.55 + (*cnt as f32 * 0.07),
                                created_at: Utc::now(),
                            });
                        }
                    }

                    // Trade detection
                    last_deaths_window.push_back((tick, victim, killer));
                    while let Some((t0, _, _)) = last_deaths_window.front().copied() {
                        if tick.saturating_sub(t0) > TRADE_WINDOW_TICKS {
                            last_deaths_window.pop_front();
                        } else {
                            break;
                        }
                    }
                    if killer != -1 && victim != -1 {
                        for (t0, prev_victim, prev_killer) in last_deaths_window.iter().copied() {
                            if prev_killer == victim
                                && tick.saturating_sub(t0) <= TRADE_WINDOW_TICKS
                            {
                                let start_tick = tick.saturating_sub(MOMENT_PAD_BEFORE);
                                let end_tick = tick + MOMENT_PAD_AFTER;
                                moments.push(KeyMoment {
                                    id: Uuid::now_v7(),
                                    match_id: match_data.id,
                                    moment_type: KeyMomentType::ImportantDuel,
                                    start_tick,
                                    end_tick,
                                    players_involved: vec![
                                        killer,
                                        victim,
                                        prev_killer,
                                        prev_victim,
                                    ],
                                    outcome: format!(
                                        "Trade: {} traded {} (prev killer {})",
                                        killer, victim, prev_killer
                                    ),
                                    importance_score: 0.65,
                                    created_at: Utc::now(),
                                });
                                break;
                            }
                        }
                    }
                }
                "bomb_planted" => {
                    current_plant_tick = Some(tick);
                }
                "bomb_defused" => {
                    if let Some(plant_tick) = current_plant_tick {
                        if postplant_ct_kills.len() >= 2 {
                            let start_tick = plant_tick.saturating_sub(MOMENT_PAD_BEFORE);
                            let end_tick = tick + MOMENT_PAD_AFTER;
                            let mut players: HashSet<i64> = HashSet::new();
                            for (_, k, v) in &postplant_ct_kills {
                                if *k != -1 {
                                    players.insert(*k);
                                }
                                if *v != -1 {
                                    players.insert(*v);
                                }
                            }
                            moments.push(KeyMoment {
                                id: Uuid::now_v7(),
                                match_id: match_data.id,
                                moment_type: KeyMomentType::Retake,
                                start_tick,
                                end_tick,
                                players_involved: players.into_iter().collect(),
                                outcome: "CT retake with defuse".to_string(),
                                importance_score: 0.8,
                                created_at: Utc::now(),
                            });
                        }
                    }
                }
                "bomb_exploded" => {
                    if let Some(plant_tick) = current_plant_tick {
                        let cluster: Vec<_> = preplant_t_kills
                            .iter()
                            .filter(|(t, _, _)| {
                                *t + EXECUTE_CLUSTER_WINDOW >= plant_tick && *t <= plant_tick
                            })
                            .cloned()
                            .collect();
                        if cluster.len() >= 2 {
                            let start_tick = plant_tick
                                .saturating_sub(EXECUTE_CLUSTER_WINDOW)
                                .saturating_sub(MOMENT_PAD_BEFORE);
                            let end_tick = tick + MOMENT_PAD_AFTER;
                            let mut players: HashSet<i64> = HashSet::new();
                            for (_, k, v) in &cluster {
                                if *k != -1 {
                                    players.insert(*k);
                                }
                                if *v != -1 {
                                    players.insert(*v);
                                }
                            }
                            moments.push(KeyMoment {
                                id: Uuid::now_v7(),
                                match_id: match_data.id,
                                moment_type: KeyMomentType::Execute,
                                start_tick,
                                end_tick,
                                players_involved: players.into_iter().collect(),
                                outcome: "T execute leading to bomb explosion".to_string(),
                                importance_score: 0.75,
                                created_at: Utc::now(),
                            });
                        }
                    }
                }
                "round_end" => {
                    // Clutch heuristic
                    if (min_alive_t == 1 && alive_ct >= 2) || (min_alive_ct == 1 && alive_t >= 2) {
                        if let Some((&player, &count)) =
                            kills_this_round.iter().max_by_key(|(_, v)| *v)
                        {
                            if count >= 3 {
                                moments.push(KeyMoment {
                                    id: Uuid::now_v7(),
                                    match_id: match_data.id,
                                    moment_type: KeyMomentType::Clutch,
                                    start_tick: round_start_tick,
                                    end_tick: tick,
                                    players_involved: vec![player],
                                    outcome: format!(
                                        "Potential clutch by {} ({} kills)",
                                        player, count
                                    ),
                                    importance_score: 0.85_f32.min(0.55 + count as f32 * 0.1),
                                    created_at: Utc::now(),
                                });
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(moments)
    }

    async fn persist_key_moments_and_behaviors(
        db: &Arc<DatabaseManager>,
        match_data: &Match,
        out: &DemoOutput,
        moments: &[KeyMoment],
    ) -> Result<()> {
        db.postgres.insert_key_moments_batch(moments).await?;

        // Build event caches via serde_json view
        #[derive(Clone)]
        struct FireEvent {
            tick: u32,
            attacker: i64,
            weapon: Option<String>,
        }
        #[derive(Clone)]
        struct HurtEvent {
            tick: u32,
            attacker: i64,
            victim: i64,
            dmg: i32,
        }
        #[derive(Clone)]
        struct DeathEvent {
            tick: u32,
            killer: i64,
            victim: i64,
        }

        let mut fires: Vec<FireEvent> = Vec::new();
        let mut hurts: Vec<HurtEvent> = Vec::new();
        let mut deaths: Vec<DeathEvent> = Vec::new();

        for e in &out.game_events {
            let tick = e.tick as u32;
            let obj = serde_json::to_value(e).unwrap_or(JsonValue::Null);
            match e.name.as_str() {
                "weapon_fire" => {
                    let attacker = get_i64(&obj, &["userid", "attacker"]).unwrap_or(-1);
                    let weapon = get_str(&obj, &["weapon"]);
                    fires.push(FireEvent {
                        tick,
                        attacker,
                        weapon,
                    });
                }
                "player_hurt" => {
                    let attacker = get_i64(&obj, &["attacker"]).unwrap_or(-1);
                    let victim = get_i64(&obj, &["userid", "victim"]).unwrap_or(-1);
                    let dmg = get_i64(&obj, &["dmg_health"]).unwrap_or(0) as i32;
                    hurts.push(HurtEvent {
                        tick,
                        attacker,
                        victim,
                        dmg,
                    });
                }
                "player_death" => {
                    let killer = get_i64(&obj, &["attacker", "killer"]).unwrap_or(-1);
                    let victim = get_i64(&obj, &["userid", "victim"]).unwrap_or(-1);
                    deaths.push(DeathEvent {
                        tick,
                        killer,
                        victim,
                    });
                }
                _ => {}
            }
        }

        // Behavior extraction per protagonist per moment
        const TICK_RATE: f32 = 64.0;
        let grenade_names: [&str; 6] = [
            "hegrenade",
            "flashbang",
            "smokegrenade",
            "molotov",
            "incgrenade",
            "decoy",
        ];

        let mut behaviors: Vec<MomentBehavior> = Vec::new();

        for m in moments {
            for steamid in &m.players_involved {
                if *steamid <= 0 {
                    continue;
                }

                // Pull snapshots window for protagonist
                let window = db
                    .timescale
                    .get_snapshots_window(
                        match_data.id,
                        *steamid,
                        m.start_tick as i32,
                        m.end_tick as i32,
                    )
                    .await?;

                if window.is_empty() {
                    continue;
                }

                // Movement/aim summaries
                let mut path_len = 0.0_f32;
                #[allow(unused_variables)]
                let mut avg_speed = 0.0_f32;
                let mut max_speed = 0.0_f32;
                let mut time_scoped_ticks = 0u32;
                let mut time_walking_ticks = 0u32;
                let mut time_air_ticks = 0u32;
                let mut flash_sum = 0.0_f32;
                let mut yaw_delta_sum = 0.0_f32;
                let mut pitch_delta_sum = 0.0_f32;

                let mut last_pos: Option<(f32, f32, f32)> = None;
                let mut last_yaw: Option<f32> = None;
                let mut last_pitch: Option<f32> = None;
                let mut speed_sum = 0.0_f32;

                for s in &window {
                    if let Some((lx, ly, lz)) = last_pos {
                        let dx = s.pos_x - lx;
                        let dy = s.pos_y - ly;
                        let dz = s.pos_z - lz;
                        path_len += (dx * dx + dy * dy + dz * dz).sqrt();
                    }
                    last_pos = Some((s.pos_x, s.pos_y, s.pos_z));

                    let speed = (s.vel_x * s.vel_x + s.vel_y * s.vel_y + s.vel_z * s.vel_z).sqrt();
                    speed_sum += speed;
                    if speed > max_speed {
                        max_speed = speed;
                    }

                    if s.is_scoped {
                        time_scoped_ticks += 1;
                    }
                    if s.is_walking {
                        time_walking_ticks += 1;
                    }
                    if s.is_airborne {
                        time_air_ticks += 1;
                    }
                    flash_sum += s.flash_duration;

                    if let Some(ly) = last_yaw {
                        yaw_delta_sum += (s.yaw - ly).abs();
                    }
                    if let Some(lp) = last_pitch {
                        pitch_delta_sum += (s.pitch - lp).abs();
                    }
                    last_yaw = Some(s.yaw);
                    last_pitch = Some(s.pitch);
                }
                avg_speed = speed_sum / (window.len() as f32);

                let duration_ticks = (m.end_tick - m.start_tick).max(1) as f32;
                let duration_seconds = duration_ticks / TICK_RATE;

                // Event-derived stats in the window
                let shots: Vec<_> = fires
                    .iter()
                    .filter(|ev| {
                        ev.attacker == *steamid && m.start_tick <= ev.tick && ev.tick <= m.end_tick
                    })
                    .collect();

                let grenades_thrown = shots
                    .iter()
                    .filter(|ev| {
                        ev.weapon
                            .as_ref()
                            .map(|w| grenade_names.iter().any(|g| g.eq_ignore_ascii_case(w)))
                            .unwrap_or(false)
                    })
                    .count() as i32;

                let hits: Vec<_> = hurts
                    .iter()
                    .filter(|ev| {
                        ev.attacker == *steamid && m.start_tick <= ev.tick && ev.tick <= m.end_tick
                    })
                    .collect();

                let dmg_dealt: i32 = hits.iter().map(|ev| ev.dmg).sum();

                let dmg_taken: i32 = hurts
                    .iter()
                    .filter(|ev| {
                        ev.victim == *steamid && m.start_tick <= ev.tick && ev.tick <= m.end_tick
                    })
                    .map(|ev| ev.dmg)
                    .sum();

                let kills_count = deaths
                    .iter()
                    .filter(|ev| {
                        ev.killer == *steamid && m.start_tick <= ev.tick && ev.tick <= m.end_tick
                    })
                    .count() as i32;

                let deaths_count = deaths
                    .iter()
                    .filter(|ev| {
                        ev.victim == *steamid && m.start_tick <= ev.tick && ev.tick <= m.end_tick
                    })
                    .count() as i32;

                let shots_fired = shots.len() as i32;
                let accuracy = if shots_fired > 0 {
                    hits.len() as f32 / shots_fired as f32
                } else {
                    0.0
                };

                // Downsample series
                let max_points = 64usize;
                let step = (window.len() as f32 / max_points as f32).ceil() as usize;
                let step = step.max(1);

                let mut series_ticks: Vec<i32> = Vec::new();
                let mut series_x: Vec<f32> = Vec::new();
                let mut series_y: Vec<f32> = Vec::new();
                let mut series_z: Vec<f32> = Vec::new();
                let mut series_yaw: Vec<f32> = Vec::new();
                let mut series_pitch: Vec<f32> = Vec::new();
                let mut series_speed: Vec<f32> = Vec::new();

                for (i, s) in window.iter().enumerate() {
                    if i % step == 0 || i + 1 == window.len() {
                        series_ticks.push(s.tick as i32);
                        series_x.push(s.pos_x);
                        series_y.push(s.pos_y);
                        series_z.push(s.pos_z);
                        let speed =
                            (s.vel_x * s.vel_x + s.vel_y * s.vel_y + s.vel_z * s.vel_z).sqrt();
                        series_speed.push(speed);
                        series_yaw.push(s.yaw);
                        series_pitch.push(s.pitch);
                    }
                }

                let features = serde_json::json!({
                    "duration_seconds": duration_seconds,
                    "path_length": path_len,
                    "avg_speed": avg_speed,
                    "max_speed": max_speed,
                    "time_scoped_seconds": time_scoped_ticks as f32 / TICK_RATE,
                    "time_walking_seconds": time_walking_ticks as f32 / TICK_RATE,
                    "time_airborne_seconds": time_air_ticks as f32 / TICK_RATE,
                    "flash_exposure_total": flash_sum,
                    "yaw_mean_abs_delta": yaw_delta_sum / (window.len().max(1) as f32),
                    "pitch_mean_abs_delta": pitch_delta_sum / (window.len().max(1) as f32),
                    "shots_fired": shots_fired,
                    "grenades_thrown": grenades_thrown,
                    "accuracy": accuracy,
                    "damage_dealt": dmg_dealt,
                    "damage_taken": dmg_taken,
                    "kills": kills_count,
                    "deaths": deaths_count,
                    "moment_type": &m.moment_type,
                    "outcome": &m.outcome,
                });

                let series = serde_json::json!({
                    "ticks": series_ticks,
                    "x": series_x,
                    "y": series_y,
                    "z": series_z,
                    "yaw": series_yaw,
                    "pitch": series_pitch,
                    "speed": series_speed,
                });

                behaviors.push(MomentBehavior {
                    id: Uuid::now_v7(),
                    match_id: match_data.id,
                    key_moment_id: m.id,
                    player_steamid: *steamid,
                    start_tick: m.start_tick as i32,
                    end_tick: m.end_tick as i32,
                    features,
                    series: Some(series),
                    created_at: Utc::now(),
                });
            }
        }

        if !behaviors.is_empty() {
            db.postgres
                .insert_moment_behaviors_batch(&behaviors)
                .await?;
        }

        Ok(())
    }
}

/* ---------------- serde_json helpers ---------------- */

fn get_i64(obj: &JsonValue, keys: &[&str]) -> Option<i64> {
    for k in keys {
        if let Some(v) = obj.get(k) {
            if let Some(x) = v.as_i64() {
                return Some(x);
            }
            if let Some(s) = v.as_str() {
                if let Ok(x) = s.parse::<i64>() {
                    return Some(x);
                }
            }
            if let Some(b) = v.as_bool() {
                return Some(if b { 1 } else { 0 });
            }
            if let Some(f) = v.as_f64() {
                return Some(f as i64);
            }
        }
    }
    None
}
fn get_str(obj: &JsonValue, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = obj.get(k) {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}
fn get_bool(obj: &JsonValue, keys: &[&str]) -> Option<bool> {
    for k in keys {
        if let Some(v) = obj.get(k) {
            if let Some(b) = v.as_bool() {
                return Some(b);
            }
            if let Some(s) = v.as_str() {
                if let Ok(b) = s.parse::<bool>() {
                    return Some(b);
                }
            }
            if let Some(i) = v.as_i64() {
                return Some(i != 0);
            }
            if let Some(f) = v.as_f64() {
                return Some(f != 0.0);
            }
        }
    }
    None
}
