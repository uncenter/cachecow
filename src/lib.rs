use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::PathBuf,
    time::{Duration, SystemTime},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Controls whether mutating operations ([`Cache::set`], [`Cache::get_or`]) automatically
/// flush the cache to disk, or leave flushing to the caller via [`Cache::flush`].
#[derive(Debug, Clone, PartialEq, Default)]
pub enum FlushPolicy {
    /// Flush to disk after every write.
    Auto,
    /// Only flush when [`Cache::flush`] is called explicitly.
    #[default]
    Manual,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Entry {
    timestamp: SystemTime,
    data: serde_json::Value,
}

#[derive(Debug)]
pub struct Cache {
    path: PathBuf,
    entries: HashMap<String, Entry>,
    entry_ttl: Duration,
    flush_policy: FlushPolicy,
}

impl Cache {
    /// Initializes a new cache. Entries are timestamped and saved to the specified path as JSON.
    /// Entries are invalidated when older than `entry_ttl`.
    pub fn new(path: PathBuf, entry_ttl: Duration, flush_policy: FlushPolicy) -> Result<Self> {
        let entries = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                    fs::create_dir_all(parent)?;
                }
                HashMap::new()
            }
        };

        Ok(Cache {
            path,
            entries,
            entry_ttl,
            flush_policy,
        })
    }

    /// Retrieve a keyed value from the cache store.
    /// Returns `None` if the entry's timestamp is older than `entry_ttl`.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.entries.get(key).and_then(|entry| {
            // TODO: Handle unwrap of future durations where behavior is ambiguous.
            let diff = SystemTime::now().duration_since(entry.timestamp).unwrap();
            if diff.lt(&self.entry_ttl) {
                serde_json::from_value(entry.data.clone()).ok()
            } else {
                None
            }
        })
    }

    /// Retrieve a keyed value from the cache store, invoking `fetch` to populate it on a miss.
    /// If [`FlushPolicy::Auto`] is set, the cache is flushed to disk after a miss.
    ///
    /// The closure may return any error type `E` as long as `E: From<`[`Error`]`>`, so that cache-internal errors can be represented within the caller's error type.
    pub fn get_or<T, F, E>(&mut self, key: &str, fetch: F) -> std::result::Result<T, E>
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce() -> std::result::Result<T, E>,
        E: From<Error>,
    {
        if let Some(data) = self.get::<T>(key) {
            return Ok(data);
        }
        let value = fetch()?;
        self.set(key, value).map_err(E::from)
    }

    /// Set a value under a key in the cache store, returning that same value.
    /// If [`FlushPolicy::Auto`] is set, the cache is flushed to disk after insertion.
    pub fn set<T: Serialize>(&mut self, key: &str, value: T) -> Result<T> {
        self.entries.insert(
            key.to_string(),
            Entry {
                timestamp: SystemTime::now(),
                data: serde_json::to_value(&value)?,
            },
        );
        if self.flush_policy == FlushPolicy::Auto {
            self.flush()?;
        }
        Ok(value)
    }

    /// Write the in-memory cache state to the filesystem.
    pub fn flush(&self) -> Result<()> {
        // TODO: Consider writing file in more atomic fashion; write to temporary file and then rename to destination, to avoid corruption in the case where the file contents are truncated and the write fails midway.
        let mut file = fs::File::create(&self.path)?;
        file.write_all(serde_json::to_string(&self.entries)?.as_bytes())?;
        Ok(())
    }

    /// Clear all entries from the cache.
    /// If [`FlushPolicy::Auto`] is set, the cache is flushed to disk after clearing.
    pub fn clear(&mut self) -> Result<()> {
        self.entries.clear();
        if self.flush_policy == FlushPolicy::Auto {
            self.flush()?;
        };
        Ok(())
    }
}
