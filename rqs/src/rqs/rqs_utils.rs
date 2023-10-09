use std::time::Duration;

static DEFAULT_BACKOFF: u32 = 1000;

pub async fn exponential_backoff<T, E>(f: impl Fn() -> Result<T, E>) -> T {
    let duration = Duration::new(0, DEFAULT_BACKOFF);
    let mut backoff = 1;
    loop {
        match f() {
            Ok(res) => return res,
            Err(_) => {
                tokio::time::sleep(backoff * duration).await;
                backoff += 1;
            }
        }
    }
}
