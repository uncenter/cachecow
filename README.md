# cachecow

A stupid simple JSON key-value filesystem cache.

## Usage

Create a cache instance:

```rs
let mut cache = cachecow::Cache::new(
    user_dirs::cache_dir()?.join("my-app").join("cache.json"), // path to cache location
    std::time::Duration::from_secs(24 * 60 * 60),              // duration in seconds for entries to remain valid
    cachecow::FlushPolicy::Auto,                               // flush to disk after every write
)?;
```

Use `FlushPolicy::Manual` to avoid writing to the filesystem on every update. You must then call `Cache::flush()` manually to preserve updated cache contents.

Set entries in the cache:

```rs
cache.set("hello", "world".to_string())?;
```

Values can be any JSON serializable value.

Retrieve entries from the cache:

```rs
cache.get::<String>("hello"); // Some("world")
```

Retrieve an entry from the cache, or set and return a default value in case of a miss:

```rs
cache.get_or("response", || -> Result<_, MyError> {
    Ok(reqwest::blocking::get("https://example.org")?.text()?)
})?;
```

Empty the cache:

```rs
cache.clear();
```

## License

[MIT](LICENSE)
