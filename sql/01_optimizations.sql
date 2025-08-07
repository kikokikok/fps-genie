-- CS2 Demo Analysis Database Initialization
-- Creates optimized schemas for high-volume professional demo data

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create optimized indexes for professional demo queries
-- These indexes support the high-volume queries described in the PDF

-- Index for finding similar professional player behaviors
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_behavior_analysis
ON player_snapshots (steamid, weapon_id, health, armor, is_alive)
WHERE is_alive = true;

-- Index for trajectory analysis (movement patterns)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_trajectory
ON player_snapshots (match_id, steamid, tick)
INCLUDE (pos_x, pos_y, pos_z, vel_x, vel_y, vel_z);

-- Index for aim analysis
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_snapshots_aim
ON player_snapshots (match_id, steamid, tick)
INCLUDE (yaw, pitch, weapon_id, is_scoped);

-- Partitioning for massive data volumes
-- Partition player_snapshots by month for optimal query performance
-- This supports the projected 100GB+ of time-series data mentioned in the PDF
SELECT add_dimension('player_snapshots', 'steamid', number_partitions => 16);

-- Create materialized views for common aggregations
CREATE MATERIALIZED VIEW IF NOT EXISTS daily_player_stats AS
SELECT
    DATE(timestamp) as date,
    steamid,
    COUNT(*) as total_ticks,
    AVG(health) as avg_health,
    AVG(armor) as avg_armor,
    COUNT(*) FILTER (WHERE is_alive) as alive_ticks,
    AVG(SQRT(vel_x^2 + vel_y^2 + vel_z^2)) as avg_velocity
FROM player_snapshots
GROUP BY DATE(timestamp), steamid;

CREATE UNIQUE INDEX ON daily_player_stats (date, steamid);

-- Refresh policy for materialized views
SELECT add_continuous_aggregate_policy('daily_player_stats',
    start_offset => INTERVAL '1 month',
    end_offset => INTERVAL '1 day',
    schedule_interval => INTERVAL '1 hour');

