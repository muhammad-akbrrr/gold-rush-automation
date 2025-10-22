use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Transaction logs table (augmented with per-chunk metadata)
    conn.execute(
        r#"
        CREATE TABLE IF NOT EXISTS transaction_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
            keeper_type TEXT NOT NULL,                -- start|settle|other
            keeper_instance_id TEXT NOT NULL,         -- hostname:pid or UUID
            op TEXT NOT NULL,                         -- operation kind, e.g. settle_bets_chunk
            round_id INTEGER,
            group_id INTEGER,
            range_start INTEGER,
            range_end INTEGER,
            transaction_signature TEXT,
            status TEXT NOT NULL,                     -- success|failed
            error_message TEXT,
            attempt INTEGER DEFAULT 0,
            retry_count INTEGER DEFAULT 0,
            backoff_ms INTEGER DEFAULT 0,
            gas_used INTEGER,
            module TEXT,
            file TEXT,
            line INTEGER,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#,
        [],
    )?;

    // Indexes for common queries
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_txlogs_ts ON transaction_logs(timestamp)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_txlogs_round ON transaction_logs(round_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_txlogs_sig ON transaction_logs(transaction_signature)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_txlogs_op_ts ON transaction_logs(op, timestamp DESC)",
        [],
    )?;

    Ok(())
}
