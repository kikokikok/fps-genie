-- CS2 Demo Analysis Database Initialization
-- Master initialization script that loads all schema components in order

\echo 'Starting CS2 Demo Analysis database initialization...'

-- Load base schema first
\i /docker-entrypoint-initdb.d/00_base_schema.sql

-- Load optimization indexes and settings
\i /docker-entrypoint-initdb.d/01_optimizations.sql

-- Load test data if available
\i /test-data/sample_data.sql

\echo 'CS2 Demo Analysis database initialization completed!'
\echo ''
\echo 'Database is ready for:'
\echo '  - Demo file processing and analysis'
\echo '  - Time-series player snapshot storage'
\echo '  - Behavioral vector storage for ML'
\echo '  - Key moment detection and analysis'
\echo '  - Player performance tracking'
\echo ''
\echo 'Connection details:'
\echo '  Database: cs2_analytics'
\echo '  User: cs2_user'
\echo '  Extensions: TimescaleDB, UUID, pg_trgm'
\echo '  Tables: tournaments, teams, players, matches, player_snapshots, key_moments, behavioral_vectors, game_events, rounds'
\echo '  Views: player_stats, match_summary'