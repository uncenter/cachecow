use cachecow::Cache;

const ONE_DAY_IN_SECONDS: u64 = 24 * 60 * 60;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cache_path = user_dirs::cache_dir()?.join("cachecow").join("cache.json");

    let mut cache = Cache::new(cache_path.clone(), false, ONE_DAY_IN_SECONDS)?;

    cache.save("hello", "world".to_string())?;
    assert_eq!(cache.get::<String>("hello"), Some("world".to_string()));

    assert_eq!(
        cache.get_or("abc", || Ok("123".to_string()))?,
        "123".to_string()
    );

    let refreshed_cache = Cache::new(cache_path, true, ONE_DAY_IN_SECONDS)?;

    assert_eq!(refreshed_cache.get::<String>("hello"), None);

    Ok(())
}
