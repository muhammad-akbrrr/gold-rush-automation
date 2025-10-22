## Project Structure

This monorepo contains two main crates:

- `keeper_lib` (shared utilities and program/DB access)
- `keepers` (cron/keeper runners as binaries).

```text
├─ abi/
│  └─ gold_rush.json                 # Program IDL/ABI for Anchor client
├─ crates/
│  ├─ keeper_lib/
│  │  └─ src/
│  │     ├─ lib.rs                   # Library root
│  │     ├─ client.rs                # Root module for client/
│  │     ├─ client/
│  │     │  ├─ anchor.rs             # Program helpers & batch ops
│  │     │  └─ rpc.rs                # RpcClient wrapper + retry/backoff
│  │     ├─ pda.rs                   # Centralized PDA derivations
│  │     ├─ storage.rs               # Storage module root
│  │     ├─ storage/
│  │     │  ├─ schema.rs             # SQLite schema
│  │     │  └─ sqlite.rs             # SQLite logger (WAL, batching)
│  │     ├─ types.rs                 # Types module root
│  │     ├─ types/
│  │     │  ├─ asset_account.rs
│  │     │  ├─ bet.rs
│  │     │  ├─ config_account.rs
│  │     │  ├─ enums.rs
│  │     │  ├─ group_asset_account.rs
│  │     │  └─ round_account.rs
│  │     └─ wallet.rs                # Keypair loader
│  └─ keepers/
│     └─ src/
│        ├─ bin/
│        │  ├─ start_round.rs        # Start-round loop
│        │  └─ settle_round.rs       # Settle-round loop
│        ├─ keepers/
│        │  ├─ start_round.rs        # Start-round logic
│        │  └─ settle_round.rs       # Settle-round logic
│        ├─ config.rs                # Runtime env loader
│        ├─ keepers.rs               # Exports submodules
│        ├─ lib.rs                   # App bootstrap (RPC, IDs, signer)
│        └─ logging.rs               # Tracing initializer
├─ data/                              # Runtime logs (SQLite; gitignored)
├─ wallets/                           # Runtime keypairs
├─ Cargo.toml                         # Workspace manifest
└─ README.md
```

Module layout: This repo uses the modern layout (e.g., root `client.rs` plus a `client/` directory for submodules) instead of the classic `client/mod.rs`. For `keepers`, we use the file-as-module `keepers.rs` which exports submodules in the `keepers/` folder (without `keepers/mod.rs`).

## Toolchain

- cargo 1.88.0 (873a06493 2025-05-10)
- rustc 1.88.0 (6b00bc388 2025-06-23)

## How to Run

1. Copy example env:

```bash
cp .env.example .env
```

2. Run a keeper:

```bash
# Start-round keeper
cargo run -p keepers --bin start_round

# Settle-round keeper
cargo run -p keepers --bin settle_round
```

Tip: for production, set `LOG_FORMAT=json` and `LOG_LEVEL=info`.

## Environment (.env)

Required:

```
SOLANA_RPC_URL=
COMMITMENT=finalized            # finalized|confirmed|processed
RPC_TIMEOUT_MS=20000
TX_MAX_RETRIES=6
PREFLIGHT=true
COMPUTE_UNIT_LIMIT=300000
PRIORITY_FEE_MICROLAMPORTS=0
BACKOFF_MS=500

KEEPER_KEYPAIR_PATH=wallets/keeper.json
TREASURY=
GOLD_PRICE_FEED_ID=
TOKEN_MINT=

START_ROUND_PERIOD_IN_SECS=30
SETTLE_ROUND_PERIOD_IN_SECS=30
MAX_REMAINING_ACCOUNTS=24

TOKEN_PROGRAM_ID=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA
ASSOCIATED_TOKEN_PROGRAM_ID=ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL
PUSH_ORACLE_PROGRAM_ID=
SYSTEM_PROGRAM_ID=11111111111111111111111111111111
PROGRAM_ID=

# Logging
LOG_LEVEL=info                  # trace|debug|info|warn|error
LOG_FORMAT=json                 # json|pretty
LOG_COLOR=false                 # pretty only

PERSIST_LOGS=true
LOG_DB_PATH=data/logs.sqlite
LOG_BATCH_MAX=200
LOG_BATCH_MS=200
LOG_QUEUE_CAP=10000
LOG_RETENTION_DAYS=90
KEEPER_INSTANCE_ID=
```

## Logging

Simple: JSON stdout for observability, SQLite for auditing per critical event (transaction chunk).

- Console: `tracing` → JSON (prod), pretty (dev). Control via `LOG_LEVEL`, `LOG_FORMAT`.
- SQLite: store critical events (success/failure) per operation-chunk; 90-day TTL; batch insert; WAL; periodic housekeeping.

Quick query (SQLite):

```sql
-- Failed in the last 24 hours for round X
SELECT timestamp, op, group_id, range_start, range_end, error_message
FROM transaction_logs
WHERE status='failed' AND round_id=? AND timestamp>=datetime('now','-1 day')
ORDER BY timestamp DESC;
```

Notes:

- SQLite files under `data/` (WAL/SHM are managed by SQLite). Directory is gitignored.
- Console info logs show high-level summaries; chunk details are at debug and persisted to SQLite.
