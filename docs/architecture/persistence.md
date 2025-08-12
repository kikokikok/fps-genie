# Persistence architecture

We split storage by access pattern:

- Relational (Postgres): match metadata, processing state, and "key moments"
- Time series (TimescaleDB): per-tick `player_snapshots` as a hypertable
- Vector (Qdrant): embeddings for similarity search over behavior snapshots/windows

This is implemented behind a single `DatabaseManager`:

- `PostgresManager`:
  - `initialize_schema()` creates `matches` and `key_moments`
  - `insert_match`, `update_match_status`, `get_unprocessed_matches`
  - `insert_key_moment` for clutches/aces/round events

- `TimescaleManager`:
  - `initialize_schema()` creates `player_snapshots` hypertable
  - `insert_snapshots_batch(&[PlayerSnapshot])` bulk-ingests with `QueryBuilder`

- `VectorManager` (Qdrant):
  - `initialize_collections()` ensures a `behavior_embeddings` collection
  - `upsert_snapshot_embeddings(match_id, round, &[PlayerSnapshot])` stores vectors with payload for later semantic queries
  - `search_similar(vector, limit)` helper to query knn

Notes:
- TimescaleDB is accessed via the same Postgres `sqlx::PgPool` for simplicity.
- The current embedding is a simple 16-dim baseline over numeric snapshot features; you can replace `embed_snapshot_16()` with a learned model later.
- If you prefer pgvector instead of Qdrant, swap `VectorManager` to a `pgvector` table with a `vector(16)` column and cosine distance index.