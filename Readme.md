# Lakebridge

Archive old PostgreSQL rows to partitioned Parquet files and query live + historical data through a single DataFusion SQL interface.

```
PostgreSQL (live OLTP)  →  Parquet (cold archive)
                 ↘              ↙
              DataFusion SQL federation
```

```bash
# Archive rows older than 30 days
lakebridge sync orders --older-than 30d

# Query across live Postgres + historical Parquet — transparently
lakebridge query "SELECT status, count(*), sum(amount) FROM orders GROUP BY status"
```

---

## Why

PostgreSQL is excellent for transactional workloads, but historical data accumulates indefinitely:

- Larger tables, larger indexes, slower vacuums
- Expensive analytical queries scanning millions of old rows
- High storage costs for data that is never modified
- Slow backups

Lakebridge moves cold data to Parquet files while keeping it fully queryable through DataFusion. No external warehouse. No Kafka. No Spark.

---

## Architecture

```
                    DataFusion SessionContext
                            │
              SQL → Logical Plan → Optimizer → Physical Plan
                            │
              ┌─────────────┴─────────────┐
              │                           │
       orders_archive               orders_live
     (ListingTable / Parquet)   (PostgresTableProvider)
              │                           │
       archive/                    tokio-postgres
         year=2026/month=07/         async query
           orders.parquet                 │
         year=2026/month=08/         RecordBatch stream
           orders.parquet
```

A logical view unifies both tables:

```sql
CREATE VIEW orders AS
  SELECT * FROM orders_archive
  UNION ALL
  SELECT * FROM orders_live;
```

---

## Use Cases

| Use Case | Description |
|----------|-------------|
| **Database archival** | Keep only recent rows in Postgres; move history to Parquet |
| **Local analytics** | Run analytical SQL over years of data without Snowflake/BigQuery |
| **Cost reduction** | Move cold data from expensive DB storage to cheap file storage |
| **Hybrid OLTP+OLAP** | Postgres for writes; Parquet + DataFusion for historical reads |
| **Data privacy** | All data stays local — no SaaS warehouse required |
| **Learning** | Reference implementation of Arrow + Parquet + DataFusion + async Postgres |

---

## Stack

| Crate | Role |
|-------|------|
| `tokio` + `tokio-postgres` | Async PostgreSQL access |
| `arrow` | In-memory columnar representation |
| `parquet` | Compressed columnar file format |
| `datafusion` | SQL query engine + federation |
| `clap` | CLI interface |
| `anyhow` | Error handling |

---

## Project Structure

```
src/
├── arrow/schema.rs          ← Schema definition, RecordBatch builders
├── archive/
│   ├── parquet_writer.rs    ← Write RecordBatches to Parquet (ZSTD)
│   ├── parquet_reader.rs    ← Read Parquet back to RecordBatches
│   └── partition.rs         ← Hive-style year=X/month=Y path generation
├── postgres/
│   ├── client.rs            ← tokio-postgres connection setup
│   ├── mapping.rs           ← PostgreSQL row → Arrow RecordBatch
│   └── provider.rs          ← Custom DataFusion TableProvider
├── datafusion/
│   ├── registry.rs          ← Register tables, EXPLAIN ANALYZE
│   ├── listing.rs           ← ListingTable over partitioned archive
│   └── context.rs           ← Build SessionContext with both tables + view
└── cli/
    ├── sync.rs              ← `lakebridge sync` command
    └── query.rs             ← `lakebridge query` command
```

---

## Key Technical Highlights

### Custom DataFusion `TableProvider` for PostgreSQL

DataFusion is extended with a `PostgresTableProvider` that fetches rows via `tokio-postgres` and converts them to Arrow `RecordBatch`es. This makes PostgreSQL and Parquet appear as equal tables to the query engine.

### Predicate Pushdown

Filters are pushed down into both table sources:
- Parquet: DataFusion uses column statistics (min/max/null_count per row group) to skip row groups entirely without reading them
- PostgreSQL: filter expressions are translated to SQL `WHERE` clauses, reducing data transferred over the wire

Inspect this with `EXPLAIN ANALYZE`:

```sql
EXPLAIN ANALYZE SELECT * FROM orders WHERE status = 'shipped' AND amount > 100
```

### Hive-Style Partitioning

Archive files are written to:
```
archive/year=2026/month=07/orders.parquet
archive/year=2026/month=08/orders.parquet
```

DataFusion's `ListingTable` reads partition columns from directory names and prunes entire directories when queries filter on `year` or `month` — without opening any files.

---

## Archive Safety

Lakebridge uses a two-phase archive process to prevent data loss:

```
1. SELECT rows WHERE created_at < cutoff
2. Write to Parquet
3. Verify Parquet (row count + checksum)
4. UPDATE orders SET archived = true WHERE id IN (...)
5. (optional, later) DELETE WHERE archived = true
```

Data is never deleted from PostgreSQL before the Parquet file is verified.

---

## Type Mapping

| PostgreSQL | Arrow |
|-----------|-------|
| `INT4` | `Int32` |
| `INT8` | `Int64` |
| `FLOAT8` | `Float64` |
| `NUMERIC` | `Decimal128(38, 10)` |
| `TEXT` / `VARCHAR` | `Utf8` |
| `BOOL` | `Boolean` |
| `TIMESTAMPTZ` | `Timestamp(Microsecond, UTC)` |
| `DATE` | `Date32` |
| `JSONB` | `Utf8` |

---

## Running Locally

```bash
# Start PostgreSQL
docker run -e POSTGRES_PASSWORD=password -p 5432:5432 postgres:16

# Archive orders older than 30 days
lakebridge sync orders --older-than 30d --dsn "postgresql://localhost/mydb"

# Query
lakebridge query "SELECT status, count(*) FROM orders GROUP BY status"
```

---

## Roadmap

- [x] Arrow schema + RecordBatch builders
- [x] Parquet write/read + round-trip tests
- [x] DataFusion SQL over Parquet
- [x] ListingTable with Hive-style partitioning
- [ ] tokio-postgres connection + type mapping
- [ ] `PostgresTableProvider` (MemoryExec phase)
- [ ] `lakebridge sync` CLI command
- [ ] `lakebridge query` CLI command
- [ ] Archive safety (manifest + checksums)
- [ ] Streaming `ExecutionPlan` for Postgres
- [ ] Filter pushdown (Inexact → Exact)
- [ ] S3/object storage via `object_store`
- [ ] Schema evolution handling
- [ ] `testcontainers` integration test suite
- [ ] Benchmarks

---

## What This Project Teaches

Building Lakebridge gives hands-on experience with:

- **Arrow**: columnar buffers, null bitmaps, zero-copy slicing, typed builders
- **Parquet**: row groups, column statistics, predicate pruning, compression
- **DataFusion**: logical/physical plans, optimizer, `TableProvider`, `ExecutionPlan`, filter pushdown
- **Async Rust**: `tokio`, `async_trait`, `Stream`, `SendableRecordBatchStream`
- **PostgreSQL internals**: wire protocol, `tokio-postgres`, prepared statements, streaming queries

---

## License

MIT
