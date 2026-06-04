use std::time::Duration;

use cachecow::{Cache, FlushPolicy};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Cache(#[from] cachecow::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
    #[error(transparent)]
    Fs(#[from] user_dirs::HomeDirError),
}

const ONE_HOUR: Duration = Duration::from_secs(60 * 60);

fn main() -> Result<(), Error> {
    let cache_path = user_dirs::cache_dir()?.join("cachecow").join("http.json");

    let mut cache = Cache::new(cache_path, ONE_HOUR, FlushPolicy::Auto)?;

    let body = cache.get_or("response", || -> Result<_, Error> {
        Ok(reqwest::blocking::get("https://httpbin.org/get")?.text()?)
    })?;

    println!("{body}");
    Ok(())
}
