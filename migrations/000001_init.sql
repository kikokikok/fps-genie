-- CS2 Demo Analysis & AI Training System - Initial schema (Postgres + TimescaleDB)

-- Extensions
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE EXTENSION IF NOT EXISTS pgcrypto;      -- for gen_random_uuid()
-- CREATE EXTENSION IF NOT EXISTS "uuid-ossp"; -- optional (uuid_generate_v4()), not used here

-- Types
DO $$ BEGIN
    CREATE TYPE processing_status AS ENUM ('pending', 'processing', 'completed', 'failed');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE key_moment_type AS ENUM ('clutch', 'ace', 'importantduel', 'ecoround', 'forcebuy', 'retake', 'execute', 'flank');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Tables (OLTP)
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

CREATE TABLE IF NOT EXISTS key_moments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    match_id UUID NOT NULL REFERENCES matches(id) ON DELETE CASCADE,
    moment_type key_moment_type NOT NULL,
    start_tick INTEGER NOT NULL,
    end_tick INTEGER NOT NULL,
    players_involved BIGINT[] NOT NULL DEFAULT '{}',
    description TEXT,
    outcome VARCHAR,
    importance_score REAL NOT NULL DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_key_moments_match ON key_moments(match_id);
CREATE INDEX IF NOT EXISTS idx_key_moments_type ON key_moments(moment_type);

-- Time-series (TimescaleDB)
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
    has_defuse_kit BOOLEAN NOT NULL DEFAULT FALSE,
    has_helmet BOOLEAN NOT NULL DEFAULT FALSE,

    money INTEGER NOT NULL DEFAULT 0,
    equipment_value INTEGER NOT NULL DEFAULT 0
);

-- Make it a hypertable (idempotent)
SELECT create_hypertable('player_snapshots', 'time', if_not_exists => TRUE);

-- Helpful indexes
CREATE INDEX IF NOT EXISTS idx_player_snapshots_match_player
    ON player_snapshots (match_id, steamid, time DESC);

CREATE INDEX IF NOT EXISTS idx_player_snapshots_round
    ON player_snapshots (match_id, round_number, time DESC);