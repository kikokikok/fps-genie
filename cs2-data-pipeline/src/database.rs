use anyhow::{anyhow, Result};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::models::{
    BehavioralEmbedding, KeyMoment, KeyMomentType, Match, MomentBehavior, PlayerSnapshot,
    ProcessingStatus,
};

/// Multi-tier DB manager for the CS2 analysis system
#[derive(Clone)]
pub struct DatabaseManager {
    pub postgres: PostgresManager,
    pub timescale: TimescaleManager,
    pub vector: VectorManager,
}

impl DatabaseManager {
    pub async fn new(postgres_url: &str, timescale_url: &str, qdrant_url: &str) -> Result<Self> {
        Ok(Self {
            postgres: PostgresManager::new(postgres_url).await?,
            timescale: TimescaleManager::new(timescale_url).await?,
            vector: VectorManager::new(qdrant_url).await?,
        })
    }
}

/* ---------------- Postgres Manager ---------------- */

#[derive(Clone)]
pub struct PostgresManager {
    pub pool: PgPool,
}

impl PostgresManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;

        // Run SQLx migrations (idempotent)
        sqlx::migrate!("../migrations").run(&pool).await?;

        Ok(PostgresManager { pool })
    }

    // Replace ad-hoc schema creation with migrations
    pub async fn initialize_schema(&self) -> Result<()> {
        sqlx::migrate!("../migrations").run(&self.pool).await?;
        Ok(())
    }

    #[allow(dead_code)]
    async fn exec(&self, sql: &str) -> Result<()> {
        sqlx::query(sql).execute(&self.pool).await?;
        Ok(())
    }

    // IMPORTANT: return the stored id (existing or newly inserted) so duplicate registration yields the same id
    pub async fn insert_match(&self, m: &Match) -> Result<Uuid> {
        let row = sqlx::query(
            r#"
            INSERT INTO matches
              (id, match_id, tournament, map_name, team1, team2, score_team1, score_team2,
               demo_file_path, demo_file_size, tick_rate, duration_seconds, created_at,
               processed_at, processing_status)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            ON CONFLICT (match_id) DO UPDATE SET match_id = EXCLUDED.match_id
            RETURNING id
            "#,
        )
        .bind(m.id)
        .bind(&m.match_id)
        .bind(&m.tournament)
        .bind(&m.map_name)
        .bind(&m.team1)
        .bind(&m.team2)
        .bind(m.score_team1)
        .bind(m.score_team2)
        .bind(&m.demo_file_path)
        .bind(m.demo_file_size)
        .bind(m.tick_rate)
        .bind(m.duration_seconds)
        .bind(m.created_at)
        .bind(m.processed_at)
        .bind(status_to_str(&m.processing_status))
        .fetch_one(&self.pool)
        .await?;
        let id: Uuid = row.try_get("id")?;
        Ok(id)
    }

    pub async fn update_match_status(
        &self,
        match_id: &str,
        status: ProcessingStatus,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE matches SET processing_status = $1, processed_at = CASE WHEN $1='completed' THEN NOW() ELSE processed_at END WHERE match_id=$2",
        )
            .bind(status_to_str(&status))
            .bind(match_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_unprocessed_matches(&self) -> Result<Vec<Match>> {
        let rows = sqlx::query(
            r#"
            SELECT id, match_id, tournament, map_name, team1, team2, score_team1, score_team2,
                   demo_file_path, demo_file_size, tick_rate, duration_seconds, created_at,
                   processed_at, processing_status
            FROM matches
            WHERE processing_status IN ('pending','failed')
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(Match {
                id: r.try_get("id")?,
                match_id: r.try_get::<String, _>("match_id")?,
                tournament: r.try_get::<Option<String>, _>("tournament")?,
                map_name: r.try_get("map_name")?,
                team1: r.try_get("team1")?,
                team2: r.try_get("team2")?,
                score_team1: r.try_get("score_team1")?,
                score_team2: r.try_get("score_team2")?,
                demo_file_path: r.try_get("demo_file_path")?,
                demo_file_size: r.try_get("demo_file_size")?,
                tick_rate: r.try_get("tick_rate")?,
                duration_seconds: r.try_get("duration_seconds")?,
                created_at: r.try_get("created_at")?,
                processed_at: r.try_get("processed_at")?,
                processing_status: str_to_status(r.try_get::<String, _>("processing_status")?)?,
            });
        }
        Ok(out)
    }

    pub async fn insert_key_moments_batch(&self, moments: &[KeyMoment]) -> Result<()> {
        for m in moments {
            sqlx::query(
                r#"
                INSERT INTO key_moments
                  (id, match_id, moment_type, start_tick, end_tick, players_involved,
                   outcome, importance_score, created_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(m.id)
            .bind(m.match_id)
            .bind(key_moment_type_to_str(&m.moment_type))
            .bind(m.start_tick as i32)
            .bind(m.end_tick as i32)
            .bind(&m.players_involved)
            .bind(&m.outcome)
            .bind(m.importance_score)
            .bind(m.created_at)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    pub async fn insert_moment_behaviors_batch(&self, behaviors: &[MomentBehavior]) -> Result<()> {
        for b in behaviors {
            sqlx::query(
                r#"
                INSERT INTO moment_behaviors
                  (id, match_id, key_moment_id, player_steamid, start_tick, end_tick,
                   features, series, created_at)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                ON CONFLICT (id) DO NOTHING
                "#,
            )
            .bind(b.id)
            .bind(b.match_id)
            .bind(b.key_moment_id)
            .bind(b.player_steamid)
            .bind(b.start_tick)
            .bind(b.end_tick)
            .bind(b.features.clone())
            .bind(b.series.clone())
            .bind(b.created_at)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}

/* ---------------- Timescale Manager ---------------- */

#[derive(Clone)]
pub struct TimescaleManager {
    pub pool: PgPool,
}

impl TimescaleManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;

        // Run SQLx migrations (idempotent)
        sqlx::migrate!("../migrations").run(&pool).await?;

        Ok(TimescaleManager { pool })
    }

    // Replace ad-hoc schema creation with migrations
    pub async fn initialize_schema(&self) -> Result<()> {
        sqlx::migrate!("../migrations").run(&self.pool).await?;
        Ok(())
    }

    // Real insert so tests can read them back
    pub async fn insert_snapshots_batch(&self, snaps: &[PlayerSnapshot]) -> Result<()> {
        if snaps.is_empty() {
            return Ok(());
        }
        for s in snaps {
            sqlx::query(
                r#"
                INSERT INTO player_snapshots
                (time, match_id, tick, steamid, round_number, health, armor,
                 pos_x, pos_y, pos_z, vel_x, vel_y, vel_z,
                 yaw, pitch, weapon_id, ammo_clip, ammo_reserve, is_alive,
                 is_airborne, is_scoped, is_walking, flash_duration, money, equipment_value)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25)
                "#,
            )
                .bind(s.timestamp)
                .bind(s.match_id)
                .bind(s.tick as i32)
                .bind(s.steamid)
                .bind(s.round_number)
                .bind(s.health)
                .bind(s.armor)
                .bind(s.pos_x)
                .bind(s.pos_y)
                .bind(s.pos_z)
                .bind(s.vel_x)
                .bind(s.vel_y)
                .bind(s.vel_z)
                .bind(s.yaw)
                .bind(s.pitch)
                .bind(s.weapon_id as i16)
                .bind(s.ammo_clip)
                .bind(s.ammo_reserve)
                .bind(s.is_alive)
                .bind(s.is_airborne)
                .bind(s.is_scoped)
                .bind(s.is_walking)
                .bind(s.flash_duration)
                .bind(s.money)
                .bind(s.equipment_value)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }

    // Method the tests call
    pub async fn get_player_snapshots(
        &self,
        match_id: Uuid,
        steamid: i64,
        limit: Option<i64>,
    ) -> Result<Vec<PlayerSnapshot>> {
        let mut sql = String::from(
            r#"
            SELECT
              time,
              match_id,
              tick,
              steamid,
              round_number,
              health,
              armor,
              pos_x,
              pos_y,
              pos_z,
              vel_x,
              vel_y,
              vel_z,
              yaw,
              pitch,
              weapon_id,
              ammo_clip,
              ammo_reserve,
              is_alive,
              is_airborne,
              is_scoped,
              is_walking,
              flash_duration,
              money,
              equipment_value
            FROM player_snapshots
            WHERE match_id = $1 AND steamid = $2
            ORDER BY tick ASC
            "#,
        );
        if limit.unwrap_or(0) > 0 {
            sql.push_str(" LIMIT $3");
        }

        let rows = if limit.unwrap_or(0) > 0 {
            sqlx::query(&sql)
                .bind(match_id)
                .bind(steamid)
                .bind(limit.unwrap())
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(&sql)
                .bind(match_id)
                .bind(steamid)
                .fetch_all(&self.pool)
                .await?
        };

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(PlayerSnapshot {
                timestamp: r.try_get("time")?,
                match_id: r.try_get("match_id")?,
                tick: r.try_get::<i32, _>("tick")? as u32,
                steamid: r.try_get("steamid")?,
                round_number: r.try_get("round_number")?,
                health: r.try_get::<f32, _>("health").unwrap_or(0.0),
                armor: r.try_get::<f32, _>("armor").unwrap_or(0.0),
                pos_x: r.try_get::<f32, _>("pos_x").unwrap_or(0.0),
                pos_y: r.try_get::<f32, _>("pos_y").unwrap_or(0.0),
                pos_z: r.try_get::<f32, _>("pos_z").unwrap_or(0.0),
                vel_x: r.try_get::<f32, _>("vel_x").unwrap_or(0.0),
                vel_y: r.try_get::<f32, _>("vel_y").unwrap_or(0.0),
                vel_z: r.try_get::<f32, _>("vel_z").unwrap_or(0.0),
                yaw: r.try_get::<f32, _>("yaw").unwrap_or(0.0),
                pitch: r.try_get::<f32, _>("pitch").unwrap_or(0.0),
                weapon_id: r.try_get::<i16, _>("weapon_id").unwrap_or(0) as u16,
                ammo_clip: r.try_get::<i32, _>("ammo_clip").unwrap_or(0),
                ammo_reserve: r.try_get::<i32, _>("ammo_reserve").unwrap_or(0),
                is_alive: r.try_get::<bool, _>("is_alive").unwrap_or(true),
                is_airborne: r.try_get::<bool, _>("is_airborne").unwrap_or(false),
                is_scoped: r.try_get::<bool, _>("is_scoped").unwrap_or(false),
                is_walking: r.try_get::<bool, _>("is_walking").unwrap_or(false),
                flash_duration: r.try_get::<f32, _>("flash_duration").unwrap_or(0.0),
                money: r.try_get::<i32, _>("money").unwrap_or(0),
                equipment_value: r.try_get::<i32, _>("equipment_value").unwrap_or(0),
            });
        }
        Ok(out)
    }

    // You can keep get_snapshots_window if needed elsewhere
    pub async fn get_snapshots_window(
        &self,
        match_id: Uuid,
        steamid: i64,
        start_tick: i32,
        end_tick: i32,
    ) -> Result<Vec<PlayerSnapshot>> {
        let rows = sqlx::query(
            r#"
            SELECT
              time,
              match_id,
              tick,
              steamid,
              round_number,
              health,
              armor,
              pos_x,
              pos_y,
              pos_z,
              vel_x,
              vel_y,
              vel_z,
              yaw,
              pitch,
              weapon_id,
              ammo_clip,
              ammo_reserve,
              is_alive,
              is_airborne,
              is_scoped,
              is_walking,
              flash_duration,
              money,
              equipment_value
            FROM player_snapshots
            WHERE match_id = $1 AND steamid = $2 AND tick BETWEEN $3 AND $4
            ORDER BY tick ASC
            "#,
        )
        .bind(match_id)
        .bind(steamid)
        .bind(start_tick)
        .bind(end_tick)
        .fetch_all(&self.pool)
        .await?;

        let mut out = Vec::with_capacity(rows.len());
        for r in rows {
            out.push(PlayerSnapshot {
                timestamp: r.try_get("time")?,
                match_id: r.try_get("match_id")?,
                tick: r.try_get::<i32, _>("tick")? as u32,
                steamid: r.try_get("steamid")?,
                round_number: r.try_get("round_number")?,
                health: r.try_get::<f32, _>("health").unwrap_or(0.0),
                armor: r.try_get::<f32, _>("armor").unwrap_or(0.0),
                pos_x: r.try_get::<f32, _>("pos_x").unwrap_or(0.0),
                pos_y: r.try_get::<f32, _>("pos_y").unwrap_or(0.0),
                pos_z: r.try_get::<f32, _>("pos_z").unwrap_or(0.0),
                vel_x: r.try_get::<f32, _>("vel_x").unwrap_or(0.0),
                vel_y: r.try_get::<f32, _>("vel_y").unwrap_or(0.0),
                vel_z: r.try_get::<f32, _>("vel_z").unwrap_or(0.0),
                yaw: r.try_get::<f32, _>("yaw").unwrap_or(0.0),
                pitch: r.try_get::<f32, _>("pitch").unwrap_or(0.0),
                weapon_id: r.try_get::<i16, _>("weapon_id").unwrap_or(0) as u16,
                ammo_clip: r.try_get::<i32, _>("ammo_clip").unwrap_or(0),
                ammo_reserve: r.try_get::<i32, _>("ammo_reserve").unwrap_or(0),
                is_alive: r.try_get::<bool, _>("is_alive").unwrap_or(true),
                is_airborne: r.try_get::<bool, _>("is_airborne").unwrap_or(false),
                is_scoped: r.try_get::<bool, _>("is_scoped").unwrap_or(false),
                is_walking: r.try_get::<bool, _>("is_walking").unwrap_or(false),
                flash_duration: r.try_get::<f32, _>("flash_duration").unwrap_or(0.0),
                money: r.try_get::<i32, _>("money").unwrap_or(0),
                equipment_value: r.try_get::<i32, _>("equipment_value").unwrap_or(0),
            });
        }
        Ok(out)
    }
}

/* ---------------- Vector Manager (Qdrant 1.15) ---------------- */

use qdrant_client::qdrant::{
    point_id::PointIdOptions, CreateCollection, Distance, PointId, PointStruct, SearchPoints,
    UpsertPoints, VectorParams, Vectors, VectorsConfig,
};
use qdrant_client::{config::QdrantConfig, Qdrant};
use serde_json::Value as JsonValue;
use tracing::{info, warn};

#[derive(Clone)]
pub struct VectorManager {
    client: Qdrant,
    // two collections: snapshots (small vector), behaviors (512-dim)
    snapshot_collection: String,
    behavior_collection: String,
}

impl VectorManager {
    pub async fn new(url: &str) -> Result<Self> {
        // qdrant-client 1.15: build config then client
        let client = Qdrant::new(QdrantConfig::from_url(url).api_key("1234"))?;
        let snapshot_collection = "snapshot_embeddings".to_string();
        let behavior_collection = "behavioral_embeddings".to_string();

        // Ensure snapshot embeddings collection (size 16 as used in pipeline embeddings)
        let create_snap = CreateCollection {
            collection_name: snapshot_collection.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: 16,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    },
                )),
            }),
            ..Default::default()
        };
        if let Err(e) = client.create_collection(create_snap).await {
            warn!("Qdrant ensure snapshot collection: {}", e);
        } else {
            info!("Qdrant collection ensured: {}", snapshot_collection);
        }

        // Ensure behavioral embeddings collection (size 512 as tests use)
        let create_beh = CreateCollection {
            collection_name: behavior_collection.clone(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(
                    VectorParams {
                        size: 512,
                        distance: Distance::Cosine.into(),
                        ..Default::default()
                    },
                )),
            }),
            ..Default::default()
        };
        if let Err(e) = client.create_collection(create_beh).await {
            warn!("Qdrant ensure behavior collection: {}", e);
        } else {
            info!("Qdrant collection ensured: {}", behavior_collection);
        }

        Ok(Self {
            client,
            snapshot_collection,
            behavior_collection,
        })
    }

    pub async fn initialize_collections(&self) -> Result<()> {
        Ok(())
    }

    // Used by pipeline snapshot embeddings (already present)
    pub async fn upsert_snapshot_embeddings(
        &self,
        match_id: Uuid,
        round_number: i32,
        snaps: &[PlayerSnapshot],
        embeddings: &[Vec<f32>],
    ) -> Result<()> {
        if snaps.is_empty() {
            return Ok(());
        }
        if snaps.len() != embeddings.len() {
            return Err(anyhow!("snapshots and embeddings mismatch"));
        }

        let mut points: Vec<PointStruct> = Vec::with_capacity(snaps.len());
        for (s, emb) in snaps.iter().zip(embeddings) {
            let id = format!("{}_{}_{}_{}", match_id, s.steamid, s.tick, round_number);
            let mut payload = serde_json::Map::<String, JsonValue>::new();
            payload.insert(
                "match_id".to_string(),
                JsonValue::String(match_id.to_string()),
            );
            payload.insert("steamid".to_string(), JsonValue::from(s.steamid));
            payload.insert("tick".to_string(), JsonValue::from(s.tick));
            payload.insert("round_number".to_string(), JsonValue::from(round_number));
            points.push(PointStruct::new(id, Vectors::from(emb.clone()), payload));
        }

        let upsert = UpsertPoints {
            collection_name: self.snapshot_collection.clone(),
            wait: Some(true),
            points,
            ..Default::default()
        };
        let _ = self.client.upsert_points(upsert).await?;
        Ok(())
    }

    // New: store behavioral embedding (matches your tests)
    pub async fn store_behavioral_vector(&self, emb: &BehavioralEmbedding) -> Result<()> {
        // Build payload
        let mut payload = serde_json::Map::<String, JsonValue>::new();
        payload.insert("match_id".into(), JsonValue::String(emb.match_id.clone()));
        payload.insert("moment_id".into(), JsonValue::String(emb.moment_id.clone()));
        payload.insert("player_steamid".into(), JsonValue::from(emb.player_steamid));
        payload.insert(
            "moment_type".into(),
            JsonValue::String(emb.moment_type.clone()),
        );
        payload.insert("metadata".into(), emb.metadata.clone());

        let point = PointStruct::new(emb.id.clone(), Vectors::from(emb.vector.clone()), payload);
        let upsert = UpsertPoints {
            collection_name: self.behavior_collection.clone(),
            wait: Some(true),
            points: vec![point],
            ..Default::default()
        };
        let _ = self.client.upsert_points(upsert).await?;
        Ok(())
    }

    // New: search behavioral embeddings
    pub async fn search_similar_behaviors(
        &self,
        embedding: &[f32],
        top: u64,
    ) -> Result<Vec<(String, f32)>> {
        let req = SearchPoints {
            collection_name: self.behavior_collection.clone(),
            vector: embedding.to_vec(),
            limit: top,
            ..Default::default()
        };
        let res = self.client.search_points(req).await?;
        let out = res
            .result
            .into_iter()
            .filter_map(|p| {
                let id_str = match p.id {
                    Some(PointId {
                        point_id_options: Some(PointIdOptions::Num(n)),
                    }) => n.to_string(),
                    Some(PointId {
                        point_id_options: Some(PointIdOptions::Uuid(u)),
                    }) => u,
                    _ => return None,
                };
                Some((id_str, p.score))
            })
            .collect();
        Ok(out)
    }

    // Existing "generic" search kept for snapshot vectors if needed
    pub async fn search_similar(&self, embedding: &[f32], top: u64) -> Result<Vec<(String, f32)>> {
        let req = SearchPoints {
            collection_name: self.snapshot_collection.clone(),
            vector: embedding.to_vec(),
            limit: top,
            ..Default::default()
        };
        let res = self.client.search_points(req).await?;
        let out = res
            .result
            .into_iter()
            .filter_map(|p| {
                let id_str = match p.id {
                    Some(PointId {
                        point_id_options: Some(PointIdOptions::Num(n)),
                    }) => n.to_string(),
                    Some(PointId {
                        point_id_options: Some(PointIdOptions::Uuid(u)),
                    }) => u,
                    _ => return None,
                };
                Some((id_str, p.score))
            })
            .collect();
        Ok(out)
    }
}

/* ---------------- helpers ---------------- */

fn status_to_str(s: &ProcessingStatus) -> &'static str {
    match *s {
        ProcessingStatus::Pending => "pending",
        ProcessingStatus::Processing => "processing",
        ProcessingStatus::Completed => "completed",
        ProcessingStatus::Failed => "failed",
    }
}

fn str_to_status(s: String) -> Result<ProcessingStatus> {
    Ok(match s.as_str() {
        "pending" => ProcessingStatus::Pending,
        "processing" => ProcessingStatus::Processing,
        "completed" => ProcessingStatus::Completed,
        "failed" => ProcessingStatus::Failed,
        other => return Err(anyhow!("unknown processing_status {other}")),
    })
}

fn key_moment_type_to_str(t: &KeyMomentType) -> &'static str {
    match *t {
        KeyMomentType::Clutch => "clutch",
        KeyMomentType::Ace => "ace",
        KeyMomentType::ImportantDuel => "importantduel",
        KeyMomentType::EcoRound => "ecoround",
        KeyMomentType::ForceBuy => "forcebuy",
        KeyMomentType::Retake => "retake",
        KeyMomentType::Execute => "execute",
        KeyMomentType::Flank => "flank",
    }
}
