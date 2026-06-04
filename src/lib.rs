use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::PathBuf,
    time::SystemTime,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Entry {
    timestamp: SystemTime,
    data: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Cache {
    path: PathBuf,
    entries: HashMap<String, Entry>,
    refresh: bool,
    entry_duration_seconds: u64,
}

impl Cache {
    /// Initializes a new cache. Entries are imestamped and saved to the specified path as JSON.
    /// Enable `refresh` to "hard refresh" the cache and always invalidate entries when requested.
    /// Entries will otherwise only be invalidated when older than the specified max duration.
    pub fn new(path: PathBuf, refresh: bool, entry_duration_seconds: u64) -> Self {
        let entries = match fs::read_to_string(&path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => {
                fs::create_dir_all(path.parent().unwrap()).unwrap();
                HashMap::new()
            }
        };

        Cache {
            path,
            entries,
            refresh,
            entry_duration_seconds,
        }
    }

    /// Retrieve a keyed value from the cache store.
    /// Returns `None` if [`Cache::refresh`] is enabled or if the entry's timestamp is older than the specified maximum duration of [`Cache::entry_duration_seconds`].
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        if self.refresh {
            return None;
        }
        self.entries.get(key).and_then(|entry| {
            let diff = SystemTime::now()
                .duration_since(entry.timestamp)
                .unwrap()
                .as_secs();
            if diff < self.entry_duration_seconds {
                serde_json::from_value(entry.data.clone()).ok()
            } else {
                None
            }
        })
    }

    /// Wrapper of the [`Cache::get`] function, accepting a closure for retrieving and then setting the value if the value is not present already or invalid.
    pub fn get_or<T, F, E>(&mut self, key: &str, fetch: F) -> Result<T, E>
    where
        T: Serialize + DeserializeOwned + Clone,
        F: FnOnce() -> Result<T, E>,
        E: From<serde_json::Error> + From<std::io::Error>,
    {
        if let Some(data) = self.get::<T>(key) {
            return Ok(data);
        }
        let value = fetch()?;
        self.set(key, value)
    }

    /// Set a value under a key to the cache store, returning that same value.
    pub fn set<T: Serialize, E: From<serde_json::Error>>(
        &mut self,
        key: &str,
        value: T,
    ) -> Result<T, E> {
        self.entries.insert(
            key.to_string(),
            Entry {
                timestamp: SystemTime::now(),
                data: serde_json::to_value(&value)?,
            },
        );
        Ok(value)
    }

    /// Set a value under a key to the cache store and immediately write the cache to the filesystem.
    /// This is a convenience wrapper for using [`Cache::set`] followed by [`Cache::write_to_file`].
    pub fn save<T, E>(&mut self, key: &str, value: T) -> Result<(), E>
    where
        T: Serialize,
        E: From<serde_json::Error> + From<std::io::Error>,
    {
        self.set::<T, E>(key, value)?;
        self.write_to_file()?;
        Ok(())
    }

    /// Save the cache to the store path (specified at cache initialization).
    fn write_to_file(&self) -> io::Result<()> {
        let mut file = fs::File::create(&self.path)?;
        file.write_all(serde_json::to_string(&self.entries)?.as_bytes())
    }
}
