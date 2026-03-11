use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Context, Result};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub(crate) struct SqliteCliDatabase {
    db_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

impl SqliteCliDatabase {
    pub(crate) fn new(db_path: impl Into<PathBuf>) -> Result<Self> {
        let db_path = db_path.into();
        if let Some(parent) = db_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create sqlite database parent directory: {}",
                    parent.display()
                )
            })?;
        }

        let database = Self {
            db_path,
            lock: Arc::new(Mutex::new(())),
        };
        database.execute("PRAGMA journal_mode=WAL;").map(|_| ())?;
        Ok(database)
    }

    pub(crate) fn execute(&self, sql: &str) -> Result<String> {
        let _guard = self
            .lock
            .lock()
            .map_err(|_| anyhow!("sqlite cli lock poisoned"))?;
        let output = Command::new("sqlite3")
            .arg(&self.db_path)
            .arg(sql)
            .output()
            .with_context(|| {
                format!(
                    "failed to execute sqlite3 against {}",
                    self.db_path.display()
                )
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(anyhow!("sqlite3 command failed: {stderr}"));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    pub(crate) fn write_temp_json(&self, prefix: &str, payload: &str) -> Result<PathBuf> {
        let temp_dir = self
            .db_path
            .parent()
            .unwrap_or(Path::new("."))
            .join(".sqlite-tmp");
        fs::create_dir_all(&temp_dir).with_context(|| {
            format!(
                "failed to create sqlite temp directory: {}",
                temp_dir.display()
            )
        })?;
        let path = temp_dir.join(format!("{prefix}-{}.json", Uuid::new_v4()));
        fs::write(&path, payload).with_context(|| {
            format!(
                "failed to write sqlite temp payload file: {}",
                path.display()
            )
        })?;
        Ok(path)
    }

    pub(crate) fn quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }
}
