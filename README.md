### Project Structure

This monorepo contains two main crates:
- `keeper_lib` (shared utilities and program/DB access)
- `keepers` (cron/keeper runners as binaries).

```text
├─ abi/
│  └─ gold_rush.json                 # Program IDL/ABI for Anchor client
├─ crates/
│  ├─ keeper_lib/                    # Library: Solana utilities, PDA, types, storage, config
│  │  ├─ src/
│  │  │  ├─ client.rs                # Root module for client submodules
│  │  │  ├─ client/                  # Client abstractions
│  │  │  │  ├─ anchor.rs             # Program/IDL helpers (optional/advanced)
│  │  │  │  └─ rpc.rs                # RpcClient wrapper + setup
│  │  │  ├─ storage/                 # Persistence (e.g., SQLite) + schema (optional)
│  │  │  ├─ types/
│  │  │  │  ├─ config.rs             # Program configuration types
│  │  │  │  ├─ enums.rs              # Domain enums (ProgramStatus, RoundStatus, ...)
│  │  │  │  └─ mod.rs                # Re-exports for types
│  │  │  ├─ pda.rs                   # Centralized PDA derivations
│  │  │  ├─ wallet.rs                # Keypair loader/validator
│  │  │  ├─ solana.rs                # Simple client setup (if not using client/rpc.rs)
│  │  │  └─ lib.rs                   # Library root (pub mod ...)
│  │  └─ Cargo.toml
│  └─ keepers/                       # Binary crate: bot/cron entrypoints
│     ├─ src/
│     │  ├─ keepers/                 # Keeper logic per role
│     │  │  ├─ start_round.rs        # Start-round keeper (run_once/run_loop)
│     │  │  └─ settle_round.rs       # Settle-round keeper (run_once/run_loop)
│     │  ├─ bin/                     # CLI entrypoints per keeper
│     │  │  ├─ start_round.rs        # CLI: run start-round keeper
│     │  │  └─ settle_round.rs       # CLI: run settle-round keeper
│     │  └─ lib.rs                   # App bootstrap (config, tracing, client, storage)
│     └─ Cargo.toml
├─ wallets/                           # Runtime keypairs
├─ Cargo.toml                         # Workspace + release profile
└─ README.md
```

Folder highlights:
- **keeper_lib**: All reusable utilities (PDA, domain types, client access, wallet). Add `storage/` for DB abstraction (a `Storage` trait + SQLite implementation) when persisting local state/logs.
- **keepers**: Keeper executors. `keepers/` holds role-specific business logic (e.g., `start_round.rs`, `settle_round.rs`), while `bin/` exposes separate CLI entrypoints for each keeper.
- **client/**: RPC/Anchor access abstraction so on-chain calls are centralized.
- **types/**: Domain types representing program state/config and related enums.
- **abi/**: Anchor IDL used to initialize `Program`/build instructions.
- **wallets/**: Runtime keypair storage.

Module layout: This repo uses the modern layout (root `client.rs` plus a `client/` directory for submodules) instead of the classic `client/mod.rs`. Keep this convention consistent across modules.

