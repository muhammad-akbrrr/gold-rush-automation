use once_cell::sync::OnceCell;
use rusqlite::{Connection, Result as SqlResult, params};
use std::cell::RefCell;
use std::fs::create_dir_all;
use std::path::Path;
use std::sync::mpsc::{SyncSender, sync_channel};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use super::schema::create_tables;

#[derive(Clone, Debug)]
pub struct SQLiteLogConfig {
    pub path: String,
    pub batch_max: usize,
    pub batch_ms: u64,
    pub queue_cap: usize,
    pub retention_days: u64,
    pub keeper_instance_id: String,
}

#[derive(Clone, Debug)]
pub struct TxLog {
    pub keeper_type: String,
    pub keeper_instance_id: String,
    pub op: String,
    pub round_id: Option<i64>,
    pub group_id: Option<i64>,
    pub range_start: Option<i64>,
    pub range_end: Option<i64>,
    pub transaction_signature: Option<String>,
    pub status: String, // success|failed
    pub error_message: Option<String>,
    pub attempt: i64,
    pub retry_count: i64,
    pub backoff_ms: i64,
    pub gas_used: Option<i64>,
    pub module: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
}

pub struct SQLiteLogger {
    tx: SyncSender<TxLog>,
    _handle: JoinHandle<()>,
}

static GLOBAL_LOGGER: OnceCell<SQLiteLogger> = OnceCell::new();
static DEFAULT_INSTANCE_ID: OnceCell<String> = OnceCell::new();

impl SQLiteLogger {
    pub fn start(cfg: SQLiteLogConfig) -> SqlResult<Self> {
        let (tx, rx) = sync_channel::<TxLog>(cfg.queue_cap);

        let handle = thread::spawn(move || {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(&cfg.path).parent() {
                let _ = create_dir_all(parent);
            }
            let mut conn = Connection::open(&cfg.path).expect("open sqlite");
            let _ = conn.pragma_update(None, "journal_mode", &"WAL");
            let _ = conn.pragma_update(None, "synchronous", &"NORMAL");
            let _ = conn.pragma_update(None, "busy_timeout", &5000);
            let _ = conn.execute("PRAGMA journal_size_limit = 104857600", []);

            create_tables(&conn).expect("create tables");

            let mut buffer: Vec<TxLog> = Vec::with_capacity(cfg.batch_max);
            let mut last_flush = Instant::now();
            let flush_interval = Duration::from_millis(cfg.batch_ms);
            let mut last_retention = Instant::now();
            let retention_interval = Duration::from_secs(3600);
            let mut last_maintenance = Instant::now();
            let maintenance_interval = Duration::from_secs(24 * 3600);

            loop {
                // Blocking recv with timeout-like behavior
                match rx.recv_timeout(Duration::from_millis(50)) {
                    Ok(item) => buffer.push(item),
                    Err(_timeout) => {}
                }

                let need_time_flush = last_flush.elapsed() >= flush_interval;
                let need_size_flush = buffer.len() >= cfg.batch_max;

                if !buffer.is_empty() && (need_time_flush || need_size_flush) {
                    let tx = conn.transaction().expect("begin tx");
                    {
                        let mut stmt = tx
                            .prepare(
                                "INSERT INTO transaction_logs (
                                    keeper_type, keeper_instance_id, op, round_id, group_id,
                                    range_start, range_end, transaction_signature, status,
                                    error_message, attempt, retry_count, backoff_ms,
                                    gas_used, module, file, line
                                ) VALUES (
                                    ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
                                )",
                            )
                            .expect("prepare insert");
                        for e in buffer.drain(..) {
                            let _ = stmt.execute(params![
                                e.keeper_type,
                                e.keeper_instance_id,
                                e.op,
                                e.round_id,
                                e.group_id,
                                e.range_start,
                                e.range_end,
                                e.transaction_signature,
                                e.status,
                                e.error_message,
                                e.attempt,
                                e.retry_count,
                                e.backoff_ms,
                                e.gas_used,
                                e.module,
                                e.file,
                                e.line,
                            ]);
                        }
                    }
                    let _ = tx.commit();
                    last_flush = Instant::now();

                    // Retention (once per hour)
                    if last_retention.elapsed() >= retention_interval {
                        let _ = conn.execute(
                            "DELETE FROM transaction_logs WHERE timestamp < datetime('now', ?)",
                            [format!("-{} days", cfg.retention_days)],
                        );
                        last_retention = Instant::now();
                    }

                    // Maintenance: checkpoint WAL and VACUUM once per day
                    if last_maintenance.elapsed() >= maintenance_interval {
                        let _ = conn.execute("PRAGMA wal_checkpoint(TRUNCATE)", []);
                        let _ = conn.execute("VACUUM", []);
                        last_maintenance = Instant::now();
                    }
                }
            }
        });

        Ok(Self {
            tx,
            _handle: handle,
        })
    }

    pub fn send(&self, entry: TxLog) -> Result<(), std::sync::mpsc::SendError<TxLog>> {
        self.tx.send(entry)
    }
}

pub fn init_global_logger(cfg: SQLiteLogConfig) {
    let logger = SQLiteLogger::start(cfg).expect("start sqlite logger");
    let _ = GLOBAL_LOGGER.set(logger);
}

pub fn log_tx(entry: TxLog) {
    if let Some(logger) = GLOBAL_LOGGER.get() {
        let mut e = entry;
        if e.keeper_instance_id.is_empty() {
            if let Some(default_id) = DEFAULT_INSTANCE_ID.get() {
                e.keeper_instance_id = default_id.clone();
            }
        }
        let _ = logger.send(e);
    }
}

pub fn is_initialized() -> bool {
    GLOBAL_LOGGER.get().is_some()
}

pub fn set_default_instance_id(id: String) {
    let _ = DEFAULT_INSTANCE_ID.set(id);
}

#[derive(Clone, Debug, Default)]
pub struct TxContext {
    pub keeper_type: String, // start|settle
    pub op: String,          // operation name
    pub round_id: Option<i64>,
    pub group_id: Option<i64>,
    pub range_start: Option<i64>,
    pub range_end: Option<i64>,
}

thread_local! {
    static CURRENT_TX_CONTEXT: RefCell<Option<TxContext>> = RefCell::new(None);
}

pub fn set_tx_context(ctx: TxContext) {
    CURRENT_TX_CONTEXT.with(|c| {
        *c.borrow_mut() = Some(ctx);
    });
}

pub fn clear_tx_context() {
    CURRENT_TX_CONTEXT.with(|c| {
        *c.borrow_mut() = None;
    });
}

pub fn get_tx_context() -> Option<TxContext> {
    CURRENT_TX_CONTEXT.with(|c| c.borrow().clone())
}
