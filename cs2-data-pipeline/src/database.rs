use sqlx::{PgPool, Row};
use qdrant_client::{Qdrant, config::QdrantConfig};
use qdrant_client::qdrant::{CreateCollection, SearchPoints, PointStruct, Vectors, Value};
use qdrant_client::qdrant::point_id;
use chrono::Utc;
use crate::models::*;
use anyhow::Result;
use uuid::Uuid;

/// Multi-tier database manager for the CS2 analysis system
#[derive(Clone)]
pub struct DatabaseManager {
    pub postgres: PostgresManager,
    pub timescale: TimescaleManager,
    pub vector: VectorManager,
}

impl DatabaseManager {
    pub async fn new(
        postgres_url: &str,
        timescale_url: &str,
        qdrant_url: &str,
    ) -> Result<Self> {
        Ok(DatabaseManager {
            postgres: PostgresManager::new(postgres_url).await?,
            timescale: TimescaleManager::new(timescale_url).await?,
            vector: VectorManager::new(qdrant_url).await?,
        })
    }
}

/// Relational database manager for match metadata
#[derive(Clone)]
pub struct PostgresManager {
    pub pool: PgPool,
}

impl PostgresManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(PostgresManager { pool })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        // Create enums first
        sqlx::query(r#"
            DO $$ BEGIN
                CREATE TYPE processing_status AS ENUM ('pending', 'processing', 'completed', 'failed');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            DO $$ BEGIN
                CREATE TYPE key_moment_type AS ENUM ('clutch', 'ace', 'importantduel', 'ecoround', 'forcebuy', 'retake', 'execute', 'flank');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
        "#).execute(&self.pool).await?;

        // Create matches table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS matches (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                match_id VARCHAR NOT NULL UNIQUE,
                tournament VARCHAR,
                map_name VARCHAR NOT NULL,
                team1 VARCHAR NOT NULL,
                team2 VARCHAR NOT NULL,
                score_team1 INTEGER NOT NULL DEFAULT 0,
                score_team2 INTEGER NOT NULL DEFAULT 0,
                demo_file_path VARCHAR NOT NULL,
                demo_file_size BIGINT NOT NULL DEFAULT 0,
                tick_rate INTEGER NOT NULL DEFAULT 64,
                duration_seconds INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                processed_at TIMESTAMPTZ,
                processing_status processing_status NOT NULL DEFAULT 'pending'
            );
            CREATE INDEX IF NOT EXISTS idx_matches_status ON matches(processing_status);
            CREATE INDEX IF NOT EXISTS idx_matches_tournament ON matches(tournament);
        "#).execute(&self.pool).await?;

        // Create players table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS players (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                steamid BIGINT NOT NULL UNIQUE,
                name VARCHAR NOT NULL,
                team VARCHAR,
                is_professional BOOLEAN NOT NULL DEFAULT false,
                rating REAL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_players_steamid ON players(steamid);
            CREATE INDEX IF NOT EXISTS idx_players_professional ON players(is_professional);
        "#).execute(&self.pool).await?;

        // Create match_participations table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS match_participations (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
                player_id UUID NOT NULL REFERENCES players(id) ON DELETE CASCADE,
                team_side VARCHAR NOT NULL,
                final_score INTEGER NOT NULL DEFAULT 0,
                kills INTEGER NOT NULL DEFAULT 0,
                deaths INTEGER NOT NULL DEFAULT 0,
                assists INTEGER NOT NULL DEFAULT 0,
                adr REAL NOT NULL DEFAULT 0.0,
                rating REAL NOT NULL DEFAULT 0.0,
                UNIQUE(match_id, player_id)
            );
            CREATE INDEX IF NOT EXISTS idx_participations_match ON match_participations(match_id);
            CREATE INDEX IF NOT EXISTS idx_participations_player ON match_participations(player_id);
        "#).execute(&self.pool).await?;

        // Create key_moments table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS key_moments (
                id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
                match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
                moment_type key_moment_type NOT NULL,
                start_tick INTEGER NOT NULL,
                end_tick INTEGER NOT NULL,
                players_involved BIGINT[] NOT NULL DEFAULT '{}',
                outcome VARCHAR,
                importance_score REAL NOT NULL DEFAULT 0.0,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            );
            CREATE INDEX IF NOT EXISTS idx_key_moments_match ON key_moments(match_id);
            CREATE INDEX IF NOT EXISTS idx_key_moments_type ON key_moments(moment_type);
        "#).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_match(&self, match_data: &Match) -> Result<Uuid> {
        let row = sqlx::query(r#"
            INSERT INTO matches (match_id, tournament, map_name, team1, team2,
                               score_team1, score_team2, demo_file_path, demo_file_size,
                               tick_rate, duration_seconds, processing_status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            ON CONFLICT (match_id) DO UPDATE SET
                demo_file_path = EXCLUDED.demo_file_path,
                demo_file_size = EXCLUDED.demo_file_size,
                processing_status = EXCLUDED.processing_status
            RETURNING id
        "#)
        .bind(&match_data.match_id)
        .bind(&match_data.tournament)
        .bind(&match_data.map_name)
        .bind(&match_data.team1)
        .bind(&match_data.team2)
        .bind(match_data.score_team1)
        .bind(match_data.score_team2)
        .bind(&match_data.demo_file_path)
        .bind(match_data.demo_file_size)
        .bind(match_data.tick_rate)
        .bind(match_data.duration_seconds)
        .bind("pending")
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    pub async fn get_unprocessed_matches(&self) -> Result<Vec<Match>> {
        let rows = sqlx::query_as::<_, Match>(r#"
            SELECT id, match_id, tournament, map_name, team1, team2,
                   score_team1, score_team2, demo_file_path, demo_file_size,
                   tick_rate, duration_seconds, processing_status, created_at, processed_at
            FROM matches
            WHERE processing_status = 'pending'
            ORDER BY created_at ASC
        "#)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows)
    }

    pub async fn update_match_status(&self, match_id: &str, status: ProcessingStatus) -> Result<()> {
        let status_str = match status {
            ProcessingStatus::Pending => "pending",
            ProcessingStatus::Processing => "processing",
            ProcessingStatus::Completed => "completed",
            ProcessingStatus::Failed => "failed",
        };

        sqlx::query(r#"
            UPDATE matches
            SET processing_status = $1, processed_at = $2
            WHERE match_id = $3
        "#)
        .bind(status_str)
        .bind(Utc::now())
        .bind(match_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

/// TimescaleDB manager for time-series player snapshots
#[derive(Clone)]
pub struct TimescaleManager {
    pub pool: PgPool,
}

impl TimescaleManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(TimescaleManager { pool })
    }

    pub async fn initialize_schema(&self) -> Result<()> {
        // Create the player_snapshots table
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS player_snapshots (
                time TIMESTAMPTZ NOT NULL,
                match_id UUID NOT NULL,
                tick INTEGER NOT NULL,
                steamid BIGINT NOT NULL,
                round_number INTEGER NOT NULL,
                health REAL NOT NULL,
                armor REAL NOT NULL,
                pos_x REAL NOT NULL,
                pos_y REAL NOT NULL,
                pos_z REAL NOT NULL,
                vel_x REAL NOT NULL,
                vel_y REAL NOT NULL,
                vel_z REAL NOT NULL,
                yaw REAL NOT NULL,
                pitch REAL NOT NULL,
                weapon_id SMALLINT NOT NULL,
                ammo_clip INTEGER NOT NULL,
                ammo_reserve INTEGER NOT NULL,
                is_alive BOOLEAN NOT NULL,
                is_airborne BOOLEAN NOT NULL,
                is_scoped BOOLEAN NOT NULL,
                is_walking BOOLEAN NOT NULL,
                flash_duration REAL NOT NULL DEFAULT 0.0,
                money INTEGER NOT NULL DEFAULT 0,
                equipment_value INTEGER NOT NULL DEFAULT 0
            );
        "#).execute(&self.pool).await?;

        // Convert to hypertable if not already
        let _ = sqlx::query(r#"
            SELECT create_hypertable('player_snapshots', 'time', if_not_exists => TRUE);
        "#).execute(&self.pool).await;

        // Create indexes for common queries
        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_snapshots_match_player
            ON player_snapshots (match_id, steamid, time DESC);
        "#).execute(&self.pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_player_snapshots_round
            ON player_snapshots (match_id, round_number, time DESC);
        "#).execute(&self.pool).await?;

        Ok(())
    }

    pub async fn insert_snapshots_batch(&self, snapshots: &[PlayerSnapshot]) -> Result<()> {
        if snapshots.is_empty() {
            return Ok(());
        }

        let mut query_builder = sqlx::QueryBuilder::new(r#"
            INSERT INTO player_snapshots (
                time, match_id, tick, steamid, round_number, health, armor,
                pos_x, pos_y, pos_z, vel_x, vel_y, vel_z,
                yaw, pitch, weapon_id, ammo_clip, ammo_reserve,
                is_alive, is_airborne, is_scoped, is_walking, flash_duration,
                money, equipment_value
            )
        "#);

        query_builder.push_values(snapshots, |mut b, snapshot| {
            b.push_bind(snapshot.timestamp)
             .push_bind(snapshot.match_id)
             .push_bind(snapshot.tick as i32)
             .push_bind(snapshot.steamid)
             .push_bind(snapshot.round_number)
             .push_bind(snapshot.health)
             .push_bind(snapshot.armor)
             .push_bind(snapshot.pos_x)
             .push_bind(snapshot.pos_y)
             .push_bind(snapshot.pos_z)
             .push_bind(snapshot.vel_x)
             .push_bind(snapshot.vel_y)
             .push_bind(snapshot.vel_z)
             .push_bind(snapshot.yaw)
             .push_bind(snapshot.pitch)
             .push_bind(snapshot.weapon_id as i16)
             .push_bind(snapshot.ammo_clip)
             .push_bind(snapshot.ammo_reserve)
             .push_bind(snapshot.is_alive)
             .push_bind(snapshot.is_airborne)
             .push_bind(snapshot.is_scoped)
             .push_bind(snapshot.is_walking)
             .push_bind(snapshot.flash_duration)
             .push_bind(snapshot.money)
             .push_bind(snapshot.equipment_value);
        });

        let query = query_builder.build();
        query.execute(&self.pool).await?;

        Ok(())
    }

    pub async fn get_player_snapshots(&self, match_id: Uuid, steamid: i64, limit: Option<i64>) -> Result<Vec<PlayerSnapshot>> {
        let limit_clause = limit.map_or("".to_string(), |l| format!("LIMIT {}", l));

        let query = format!(r#"
            SELECT * FROM player_snapshots
            WHERE match_id = $1 AND steamid = $2
            ORDER BY time DESC {}
        "#, limit_clause);

        let rows = sqlx::query(&query)
            .bind(match_id)
            .bind(steamid)
            .fetch_all(&self.pool)
            .await?;

        let snapshots = rows.into_iter().map(|row| PlayerSnapshot {
            timestamp: row.get("time"),
            match_id: row.get("match_id"),
            tick: row.get::<i32, _>("tick") as u32,
            steamid: row.get("steamid"),
            round_number: row.get("round_number"),
            health: row.get("health"),
            armor: row.get("armor"),
            pos_x: row.get("pos_x"),
            pos_y: row.get("pos_y"),
            pos_z: row.get("pos_z"),
            vel_x: row.get("vel_x"),
            vel_y: row.get("vel_y"),
            vel_z: row.get("vel_z"),
            yaw: row.get("yaw"),
            pitch: row.get("pitch"),
            weapon_id: row.get::<i16, _>("weapon_id") as u16,
            ammo_clip: row.get("ammo_clip"),
            ammo_reserve: row.get("ammo_reserve"),
            is_alive: row.get("is_alive"),
            is_airborne: row.get("is_airborne"),
            is_scoped: row.get("is_scoped"),
            is_walking: row.get("is_walking"),
            flash_duration: row.get("flash_duration"),
            money: row.get("money"),
            equipment_value: row.get("equipment_value"),
        }).collect();

        Ok(snapshots)
    }
}

/// Qdrant vector database manager for behavioral embeddings
#[derive(Clone)]
pub struct VectorManager {
    client: Qdrant,
}

impl VectorManager {
    pub async fn new(qdrant_url: &str) -> Result<Self> {
        let config = QdrantConfig::from_url(qdrant_url);
        let client = Qdrant::new(config)?;
        Ok(VectorManager { client })
    }

    pub async fn initialize_collections(&self) -> Result<()> {
        use qdrant_client::qdrant::{VectorParams, VectorsConfig};

        let create_collection = CreateCollection {
            collection_name: "behavioral_vectors".to_string(),
            vectors_config: Some(VectorsConfig {
                config: Some(qdrant_client::qdrant::vectors_config::Config::Params(VectorParams {
                    size: 512, // Configurable embedding dimension
                    distance: qdrant_client::qdrant::Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        };

        let _ = self.client.create_collection(create_collection).await;
        Ok(())
    }

    pub async fn store_behavioral_vector(&self, embedding: &BehavioralEmbedding) -> Result<()> {
        let point = PointStruct {
            id: Some(qdrant_client::qdrant::PointId {
                point_id_options: Some(point_id::PointIdOptions::Uuid(embedding.id.clone())),
            }),
            vectors: Some(Vectors {
                vectors_options: Some(qdrant_client::qdrant::vectors::VectorsOptions::Vector(
                    qdrant_client::qdrant::Vector {
                        data: embedding.vector.clone(),
                        indices: None,
                        vectors_count: None,
                        vector: Some(qdrant_client::qdrant::vector::Vector::Dense(qdrant_client::qdrant::DenseVector {
                            data: embedding.vector.clone(),
                        })),
                    }
                )),
            }),
            payload: {
                let mut payload = std::collections::HashMap::new();
                payload.insert("match_id".to_string(), Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::StringValue(embedding.match_id.clone()))
                });
                payload.insert("player_steamid".to_string(), Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::IntegerValue(embedding.player_steamid))
                });
                payload.insert("moment_type".to_string(), Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::StringValue(embedding.moment_type.clone()))
                });
                payload.insert("moment_id".to_string(), Value {
                    kind: Some(qdrant_client::qdrant::value::Kind::StringValue(embedding.moment_id.clone()))
                });
                payload
            },
        };

        let points = vec![point];

        let upsert_points = qdrant_client::qdrant::UpsertPoints {
            collection_name: "behavioral_vectors".to_string(),
            points,
            ..Default::default()
        };

        self.client.upsert_points(upsert_points).await?;
        Ok(())
    }

    pub async fn search_similar_behaviors(&self, query_vector: &[f32], limit: usize) -> Result<Vec<String>> {
        let search_points = SearchPoints {
            collection_name: "behavioral_vectors".to_string(),
            vector: query_vector.to_vec(),
            limit: limit as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        };

        let response = self.client.search_points(search_points).await?;

        let ids = response.result.into_iter().map(|point| {
            match point.id.and_then(|id| id.point_id_options) {
                Some(point_id::PointIdOptions::Uuid(s)) => s,
                Some(point_id::PointIdOptions::Num(n)) => n.to_string(),
                None => "unknown".to_string(),
            }
        }).collect();

        Ok(ids)
    }
}
