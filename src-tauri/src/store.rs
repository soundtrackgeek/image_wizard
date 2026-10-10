use crate::model::{Artwork, Operation, Preferences};
use rusqlite::{params, Connection};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{SystemTime, UNIX_EPOCH},
};

type HardwareCache = (Option<String>, Option<u64>, u64);

pub struct Store {
    pub root: PathBuf,
    _instance_lock: std::fs::File,
    pub connection: Mutex<Connection>,
    pub busy: AtomicBool,
    pub cancelled: AtomicBool,
    pub hardware: Mutex<Option<HardwareCache>>,
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl Store {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        let instance_lock = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(root.join("instance.lock"))
            .map_err(|e| e.to_string())?;
        instance_lock
            .try_lock()
            .map_err(|_| "Image Wizard is already running. Use the existing window.".to_string())?;
        for name in ["inputs", "outputs", "operations", "runtime", "logs"] {
            std::fs::create_dir_all(root.join(name)).map_err(|e| e.to_string())?;
        }
        let connection =
            Connection::open(root.join("studio.sqlite3")).map_err(|e| e.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL); CREATE TABLE IF NOT EXISTS artworks (id TEXT PRIMARY KEY, metadata TEXT NOT NULL, created_at INTEGER NOT NULL);").map_err(|e| e.to_string())?;
        let store = Self {
            root,
            _instance_lock: instance_lock,
            connection: Mutex::new(connection),
            busy: AtomicBool::new(false),
            cancelled: AtomicBool::new(false),
            hardware: Mutex::new(None),
        };
        if let Some(mut operation) = store.get::<Operation>("operation")? {
            if operation.finished_at.is_none() {
                operation.phase = "interrupted".into();
                operation.detail = "The app closed during this operation. Downloads can be resumed; generation can be retried.".into();
                if ["generate", "check"].contains(&operation.kind.as_str()) {
                    if let Some(artwork) = store.recover_artwork(&operation.id) {
                        store.add_artwork(&artwork)?;
                        operation.phase = "succeeded".into();
                        operation.detail = "Recovered the completed image after restart. It is available in your library.".into();
                    }
                }
                operation.finished_at = Some(now());
                store.put("operation", &operation)?;
            }
        }
        Ok(store)
    }
    fn recover_artwork(&self, id: &str) -> Option<Artwork> {
        let path = self.root.join("outputs").join(format!("{id}.png"));
        let data = std::fs::read(path.with_extension("png.metadata.json")).ok()?;
        let artwork: Artwork = serde_json::from_slice(&data).ok()?;
        if artwork.id != id || Path::new(&artwork.path) != path || !path.is_file() {
            return None;
        }
        let image = image::open(&path).ok()?;
        if image.width() != artwork.width || image.height() != artwork.height {
            return None;
        }
        Some(artwork)
    }
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let db = self.connection.lock().map_err(|e| e.to_string())?;
        let mut query = db
            .prepare("SELECT value FROM settings WHERE key=?1")
            .map_err(|e| e.to_string())?;
        let mut rows = query.query([key]).map_err(|e| e.to_string())?;
        rows.next()
            .map_err(|e| e.to_string())?
            .map(|row| {
                let json: String = row.get(0).map_err(|e| e.to_string())?;
                serde_json::from_str(&json).map_err(|e| e.to_string())
            })
            .transpose()
    }
    pub fn put<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        self.connection.lock().map_err(|e| e.to_string())?.execute("INSERT INTO settings(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, serde_json::to_string(value).map_err(|e| e.to_string())?]).map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn preferences(&self) -> Result<Preferences, String> {
        Ok(self.get("preferences")?.unwrap_or_else(|| Preferences {
            model_dir: self
                .root
                .join("models")
                .join("qwen-image-2.1-turbo")
                .to_string_lossy()
                .into(),
            profile: "nf4".into(),
            license_accepted: false,
            checked_profile: None,
        }))
    }
    pub fn artworks(&self) -> Result<Vec<Artwork>, String> {
        let db = self.connection.lock().map_err(|e| e.to_string())?;
        let mut query = db
            .prepare("SELECT metadata FROM artworks ORDER BY created_at DESC LIMIT 1000")
            .map_err(|e| e.to_string())?;
        let rows = query
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(|e| e.to_string())?;
        rows.map(|row| {
            serde_json::from_str(&row.map_err(|e| e.to_string())?).map_err(|e| e.to_string())
        })
        .collect()
    }
    pub fn add_artwork(&self, artwork: &Artwork) -> Result<(), String> {
        self.connection
            .lock()
            .map_err(|e| e.to_string())?
            .execute(
                "INSERT OR IGNORE INTO artworks(id,metadata,created_at) VALUES(?1,?2,?3)",
                params![
                    artwork.id,
                    serde_json::to_string(artwork).map_err(|e| e.to_string())?,
                    artwork.created_at
                ],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    pub fn begin(&self, kind: &str) -> Result<Operation, String> {
        if self
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(
                "An operation is already running. Wait for it to finish or stop it first.".into(),
            );
        }
        self.cancelled.store(false, Ordering::SeqCst);
        let operation = Operation {
            id: uuid::Uuid::new_v4().to_string(),
            kind: kind.into(),
            phase: "starting".into(),
            detail: "Preparing…".into(),
            completed: None,
            total: None,
            started_at: now(),
            finished_at: None,
        };
        if let Err(error) = self.put("operation", &operation) {
            self.busy.store(false, Ordering::SeqCst);
            return Err(error);
        }
        Ok(operation)
    }
    pub fn finish(&self, mut operation: Operation, outcome: Result<(), String>) {
        operation.finished_at = Some(now());
        match outcome {
            Ok(()) => {
                operation.phase = "succeeded".into();
            }
            Err(error) => {
                operation.phase = if self.cancelled.load(Ordering::SeqCst) {
                    "cancelled"
                } else {
                    "failed"
                }
                .into();
                operation.detail = error;
            }
        }
        let _ = self.put("operation", &operation);
        self.busy.store(false, Ordering::SeqCst);
    }
    pub fn owned_image(&self, raw: &str) -> Result<PathBuf, String> {
        let target = Path::new(raw).canonicalize().map_err(|e| e.to_string())?;
        for folder in ["inputs", "outputs"] {
            let allowed = self
                .root
                .join(folder)
                .canonicalize()
                .map_err(|e| e.to_string())?;
            if target.starts_with(allowed) && target.is_file() {
                return Ok(target);
            }
        }
        Err("This image is outside the studio's image folders. Import it first.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_marks_active_work_interrupted() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().into()).unwrap();
        store.begin("install").unwrap();
        drop(store);
        let reopened = Store::new(dir.path().into()).unwrap();
        assert_eq!(
            reopened
                .get::<Operation>("operation")
                .unwrap()
                .unwrap()
                .phase,
            "interrupted"
        );
        assert!(reopened.begin("install").is_ok());
    }
    #[test]
    fn second_instance_cannot_interrupt_the_first() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().into()).unwrap();
        store.begin("install").unwrap();
        assert!(Store::new(dir.path().into()).is_err());
        assert!(store
            .get::<Operation>("operation")
            .unwrap()
            .unwrap()
            .finished_at
            .is_none());
    }
    #[test]
    fn outside_image_is_rejected_and_jobs_serialize() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        let store = Store::new(dir.path().into()).unwrap();
        assert!(store.owned_image(outside.path().to_str().unwrap()).is_err());
        store.begin("install").unwrap();
        assert!(store.begin("generate").is_err());
    }
    #[test]
    fn completed_image_is_recovered_once_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().into()).unwrap();
        let operation = store.begin("generate").unwrap();
        let path = dir
            .path()
            .join("outputs")
            .join(format!("{}.png", operation.id));
        image::RgbaImage::new(2, 2).save(&path).unwrap();
        let artwork = Artwork {
            id: operation.id,
            path: path.to_string_lossy().into(),
            prompt: "a sphere".into(),
            effective_prompt: "a sphere".into(),
            mode: "create".into(),
            width: 2,
            height: 2,
            seed: 42,
            has_alpha: true,
            transparency_requested: false,
            profile: "nf4".into(),
            created_at: now(),
            references: vec![],
            parent_id: None,
            provenance: serde_json::Value::Null,
        };
        std::fs::write(
            path.with_extension("png.metadata.json"),
            serde_json::to_vec(&artwork).unwrap(),
        )
        .unwrap();
        drop(store);
        let reopened = Store::new(dir.path().into()).unwrap();
        assert_eq!(reopened.artworks().unwrap().len(), 1);
        assert_eq!(
            reopened
                .get::<Operation>("operation")
                .unwrap()
                .unwrap()
                .phase,
            "succeeded"
        );
        drop(reopened);
        assert_eq!(
            Store::new(dir.path().into())
                .unwrap()
                .artworks()
                .unwrap()
                .len(),
            1
        );
    }
}
