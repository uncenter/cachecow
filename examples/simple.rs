use std::time::Duration;

use cachecow::{Cache, FlushPolicy};

const ONE_DAY: Duration = Duration::from_secs(24 * 60 * 60);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = user_dirs::cache_dir()?.join("cachecow").join("cache.json");

    let mut cache = Cache::new(cache_path.clone(), false, ONE_DAY, FlushPolicy::Auto)?;

    cache.set("hello", "world".to_string())?;
    assert_eq!(cache.get::<String>("hello"), Some("world".to_string()));

    assert_eq!(
        cache.get_or("abc", || Ok("123".to_string()))?,
        "123".to_string()
    );

    let refreshed_cache = Cache::new(cache_path, true, ONE_DAY, FlushPolicy::Auto)?;

    assert_eq!(refreshed_cache.get::<String>("hello"), None);

    Ok(())
}
