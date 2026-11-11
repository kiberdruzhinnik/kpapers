use sqlx::SqlitePool;
use std::{
    cell::RefCell,
    fs,
    future::Future,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex, Once},
};
use tracing_subscriber::fmt::MakeWriter;

pub async fn database() -> (SqlitePool, PathBuf) {
    let path = std::env::temp_dir().join(format!("kpapers-test-{}.sqlite", uuid::Uuid::new_v4()));
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let pool = crate::db::connect_and_migrate(&url)
        .await
        .expect("create isolated test database");
    (pool, path)
}

pub async fn cleanup(pool: SqlitePool, path: PathBuf) {
    pool.close().await;
    for suffix in ["", "-wal", "-shm"] {
        let mut file = path.as_os_str().to_os_string();
        file.push(suffix);
        let _ = fs::remove_file(PathBuf::from(file));
    }
}

thread_local! {
    static LOG_BUFFER: RefCell<Option<Arc<Mutex<Vec<u8>>>>> = const { RefCell::new(None) };
}

/// Run one future and return the JSON log lines emitted on its thread.
///
/// Tests run in parallel, so the subscriber is global but writes to a thread-local buffer.
pub async fn capture_logs<F, T>(future: F) -> (T, String)
where
    F: Future<Output = T>,
{
    init_test_subscriber();
    let buffer = Arc::new(Mutex::new(Vec::new()));
    LOG_BUFFER.with(|slot| *slot.borrow_mut() = Some(buffer.clone()));
    let result = future.await;
    LOG_BUFFER.with(|slot| *slot.borrow_mut() = None);
    let logs = match Arc::try_unwrap(buffer) {
        Ok(mutex) => mutex.into_inner().unwrap(),
        Err(buffer) => buffer.lock().unwrap().clone(),
    };
    (result, String::from_utf8(logs).unwrap())
}

fn init_test_subscriber() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_target(false)
            .with_writer(ThreadLocalWriter)
            .finish();
        let _ = tracing::subscriber::set_global_default(subscriber);
    });
}

pub fn json_events(logs: &str) -> Vec<serde_json::Value> {
    logs.lines()
        .map(|line| serde_json::from_str(line).expect("log line is valid JSON"))
        .collect()
}

struct ThreadLocalWriter;

impl Write for ThreadLocalWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        LOG_BUFFER.with(|slot| {
            if let Some(buffer) = slot.borrow().as_ref() {
                buffer.lock().unwrap().extend_from_slice(bytes);
            }
        });
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for ThreadLocalWriter {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        Self
    }
}
