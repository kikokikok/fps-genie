-- CS2 Demo Analysis Database Schema
-- Base schema for CS2 demo analysis and AI training system

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- =============================================================================
-- Base Tables
-- =============================================================================

-- Tournaments
CREATE TABLE IF NOT EXISTS tournaments (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    prize_pool INTEGER,
    tier CHAR(1) CHECK (tier IN ('S', 'A', 'B', 'C')),
    location TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Teams
CREATE TABLE IF NOT EXISTS teams (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    region TEXT,
    current_ranking INTEGER,
    founded_date DATE,
    website TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Players
CREATE TABLE IF NOT EXISTS players (
    id SERIAL PRIMARY KEY,
    steam_id BIGINT NOT NULL UNIQUE,
    nickname TEXT NOT NULL,
    real_name TEXT,
    team_id INTEGER REFERENCES teams(id),
    role TEXT CHECK (role IN ('entry_fragger', 'support', 'awper', 'igl', 'lurker', 'rifler')),
    country_code CHAR(2),
    birth_date DATE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Matches
CREATE TABLE IF NOT EXISTS matches (
    id SERIAL PRIMARY KEY,
    match_id TEXT NOT NULL UNIQUE,
    tournament_id INTEGER REFERENCES tournaments(id),
    team1_id INTEGER REFERENCES teams(id),
    team2_id INTEGER REFERENCES teams(id),
    map_name TEXT NOT NULL,
    match_date TIMESTAMPTZ NOT NULL,
    team1_score INTEGER DEFAULT 0,
    team2_score INTEGER DEFAULT 0,
    demo_file_path TEXT,
    demo_file_size BIGINT,
    processing_status TEXT DEFAULT 'pending' CHECK (processing_status IN ('pending', 'processing', 'completed', 'failed')),
    processing_started_at TIMESTAMPTZ,
    processing_completed_at TIMESTAMPTZ,
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Time-series Tables (for TimescaleDB)
-- =============================================================================

-- Player snapshots (high-frequency time-series data)
CREATE TABLE IF NOT EXISTS player_snapshots (
    id BIGSERIAL,
    match_id TEXT NOT NULL,
    steamid BIGINT NOT NULL,
    tick INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    round_number INTEGER,
    
    -- Position and movement
    pos_x REAL,
    pos_y REAL,
    pos_z REAL,
    vel_x REAL,
    vel_y REAL,
    vel_z REAL,
    yaw REAL,
    pitch REAL,
    
    -- Player state
    health INTEGER,
    armor INTEGER,
    is_alive BOOLEAN,
    is_defusing BOOLEAN,
    is_planting BOOLEAN,
    is_reloading BOOLEAN,
    is_scoped BOOLEAN,
    is_walking BOOLEAN,
    is_ducking BOOLEAN,
    
    -- Weapon and equipment
    weapon_id INTEGER,
    weapon_name TEXT,
    ammo_clip INTEGER,
    ammo_reserve INTEGER,
    has_helmet BOOLEAN,
    has_defuser BOOLEAN,
    
    -- Economy
    money INTEGER,
    equipment_value INTEGER,
    
    -- Additional metadata
    metadata JSONB,
    
    PRIMARY KEY (timestamp, steamid, match_id)
);

-- Convert to TimescaleDB hypertable
SELECT create_hypertable('player_snapshots', 'timestamp', if_not_exists => TRUE);

-- Key moments (clutches, aces, multi-kills, etc.)
CREATE TABLE IF NOT EXISTS key_moments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    match_id TEXT NOT NULL,
    round_number INTEGER NOT NULL,
    tick INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    moment_type TEXT NOT NULL, -- 'ace', 'clutch_1v2', 'clutch_1v3', 'multi_kill', 'entry_frag', etc.
    player_steam_id BIGINT NOT NULL,
    description TEXT,
    significance_score REAL CHECK (significance_score >= 0 AND significance_score <= 1),
    context JSONB, -- Additional context like enemies_alive, bomb_planted, etc.
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Behavioral vectors for ML
CREATE TABLE IF NOT EXISTS behavioral_vectors (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    match_id TEXT NOT NULL,
    player_steam_id BIGINT NOT NULL,
    tick INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    vector_type TEXT NOT NULL, -- 'movement', 'aim', 'positioning', 'decision_making', etc.
    embedding REAL[] NOT NULL, -- Vector embedding
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- =============================================================================
-- Game Events
-- =============================================================================

-- Game events (deaths, bomb events, etc.)
CREATE TABLE IF NOT EXISTS game_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    match_id TEXT NOT NULL,
    tick INTEGER NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    event_type TEXT NOT NULL,
    player_steam_id BIGINT,
    target_steam_id BIGINT,
    weapon_name TEXT,
    headshot BOOLEAN,
    position_x REAL,
    position_y REAL,
    position_z REAL,
    event_data JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Round data
CREATE TABLE IF NOT EXISTS rounds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    match_id TEXT NOT NULL,
    round_number INTEGER NOT NULL,
    start_tick INTEGER NOT NULL,
    end_tick INTEGER,
    start_timestamp TIMESTAMPTZ NOT NULL,
    end_timestamp TIMESTAMPTZ,
    winner_team_id INTEGER REFERENCES teams(id),
    winner_side TEXT CHECK (winner_side IN ('CT', 'T')),
    end_reason TEXT, -- 'bomb_exploded', 'bomb_defused', 'eliminated', 'time_expired'
    ct_score INTEGER,
    t_score INTEGER,
    round_type TEXT, -- 'pistol', 'eco', 'force_buy', 'full_buy'
    metadata JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    
    UNIQUE(match_id, round_number)
);

-- =============================================================================
-- Indexes for Performance
-- =============================================================================

-- Primary lookup indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_player_snapshots_match_steamid 
ON player_snapshots (match_id, steamid);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_player_snapshots_steamid_timestamp 
ON player_snapshots (steamid, timestamp);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_key_moments_match_player 
ON key_moments (match_id, player_steam_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_key_moments_type_significance 
ON key_moments (moment_type, significance_score DESC);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_behavioral_vectors_match_player 
ON behavioral_vectors (match_id, player_steam_id);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_behavioral_vectors_type 
ON behavioral_vectors (vector_type);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_game_events_match_tick 
ON game_events (match_id, tick);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_game_events_player_type 
ON game_events (player_steam_id, event_type);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_rounds_match 
ON rounds (match_id, round_number);

-- Full-text search indexes
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_players_nickname_trgm 
ON players USING gin (nickname gin_trgm_ops);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_teams_name_trgm 
ON teams USING gin (name gin_trgm_ops);

-- JSONB indexes for metadata queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_matches_metadata 
ON matches USING gin (metadata);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_player_snapshots_metadata 
ON player_snapshots USING gin (metadata);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_key_moments_context 
ON key_moments USING gin (context);

-- =============================================================================
-- Views for Common Queries
-- =============================================================================

-- Player statistics view
CREATE OR REPLACE VIEW player_stats AS
SELECT 
    p.steam_id,
    p.nickname,
    p.real_name,
    t.name as team_name,
    COUNT(DISTINCT m.match_id) as matches_played,
    COUNT(km.id) as total_key_moments,
    COUNT(km.id) FILTER (WHERE km.moment_type LIKE 'clutch%') as clutches,
    COUNT(km.id) FILTER (WHERE km.moment_type = 'ace') as aces,
    COUNT(km.id) FILTER (WHERE km.moment_type = 'entry_frag') as entry_frags,
    ROUND(AVG(km.significance_score), 3) as avg_significance,
    MAX(km.created_at) as last_played
FROM players p
LEFT JOIN teams t ON p.team_id = t.id
LEFT JOIN key_moments km ON p.steam_id = km.player_steam_id
LEFT JOIN matches m ON km.match_id = m.match_id
WHERE p.is_active = true
GROUP BY p.steam_id, p.nickname, p.real_name, t.name;

-- Match summary view
CREATE OR REPLACE VIEW match_summary AS
SELECT 
    m.match_id,
    m.map_name,
    m.match_date,
    t1.name as team1_name,
    t2.name as team2_name,
    m.team1_score,
    m.team2_score,
    m.processing_status,
    COUNT(km.id) as total_key_moments,
    COUNT(ps.id) as total_snapshots,
    COUNT(ge.id) as total_events
FROM matches m
LEFT JOIN teams t1 ON m.team1_id = t1.id
LEFT JOIN teams t2 ON m.team2_id = t2.id
LEFT JOIN key_moments km ON m.match_id = km.match_id
LEFT JOIN player_snapshots ps ON m.match_id = ps.match_id
LEFT JOIN game_events ge ON m.match_id = ge.match_id
GROUP BY m.match_id, m.map_name, m.match_date, t1.name, t2.name, 
         m.team1_score, m.team2_score, m.processing_status;

-- =============================================================================
-- Functions and Triggers
-- =============================================================================

-- Update timestamp trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add update triggers to tables that need them
CREATE TRIGGER update_tournaments_updated_at BEFORE UPDATE ON tournaments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_teams_updated_at BEFORE UPDATE ON teams
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_players_updated_at BEFORE UPDATE ON players
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_matches_updated_at BEFORE UPDATE ON matches
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to get player performance metrics
CREATE OR REPLACE FUNCTION get_player_performance(player_steamid BIGINT, days_back INTEGER DEFAULT 30)
RETURNS TABLE (
    metric_name TEXT,
    metric_value NUMERIC,
    metric_description TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        'total_matches'::TEXT,
        COUNT(DISTINCT km.match_id)::NUMERIC,
        'Total matches played in the last ' || days_back || ' days'::TEXT
    FROM key_moments km
    WHERE km.player_steam_id = player_steamid
    AND km.created_at >= NOW() - INTERVAL '1 day' * days_back
    
    UNION ALL
    
    SELECT 
        'key_moments'::TEXT,
        COUNT(km.id)::NUMERIC,
        'Total key moments in the last ' || days_back || ' days'::TEXT
    FROM key_moments km
    WHERE km.player_steam_id = player_steamid
    AND km.created_at >= NOW() - INTERVAL '1 day' * days_back
    
    UNION ALL
    
    SELECT 
        'avg_significance'::TEXT,
        ROUND(AVG(km.significance_score), 3)::NUMERIC,
        'Average significance score in the last ' || days_back || ' days'::TEXT
    FROM key_moments km
    WHERE km.player_steam_id = player_steamid
    AND km.created_at >= NOW() - INTERVAL '1 day' * days_back;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Initial Data Setup
-- =============================================================================

-- Insert default data
INSERT INTO tournaments (name, start_date, end_date, prize_pool, tier) VALUES
('Development Tournament', '2024-01-01', '2024-01-31', 100000, 'A')
ON CONFLICT DO NOTHING;

INSERT INTO teams (name, region, current_ranking) VALUES
('Development Team 1', 'Global', 1),
('Development Team 2', 'Global', 2)
ON CONFLICT DO NOTHING;

-- Grant permissions for application user
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO cs2_user;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO cs2_user;
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA public TO cs2_user;

-- Grant TimescaleDB specific permissions
GRANT ALL ON SCHEMA public TO cs2_user;

-- Success message
DO $$
BEGIN
    RAISE NOTICE 'CS2 Demo Analysis database schema initialized successfully!';
    RAISE NOTICE 'TimescaleDB extensions enabled';
    RAISE NOTICE 'All tables, indexes, views, and functions created';
    RAISE NOTICE 'Ready for demo data ingestion and analysis';
END $$;