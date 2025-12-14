use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tokio::time::{sleep, Duration};

#[derive(Clone, Debug)]
pub struct CacheItem<T: Clone + Send + Sync + 'static> {
    value: T,
    expires_at: Instant,
}

#[derive(Clone)]
pub struct AsyncCache<T: Clone + Send + Sync + 'static> {
    inner: Arc<Mutex<HashMap<String, CacheItem<T>>>>,
}

impl<T: Clone + Send + Sync + 'static> AsyncCache<T> {
    pub fn new(cleanup_interval_secs: u64) -> Self {
        let inner = Arc::new(Mutex::new(HashMap::<String, CacheItem<T>>::new()));

        let inner_clone = inner.clone();

        tokio::spawn(async move {
            let interval = Duration::from_secs(cleanup_interval_secs);

            loop {
                sleep(interval).await;

                let mut map = inner_clone.lock().unwrap();

                let now = Instant::now();

                map.retain(|_, item| item.expires_at > now);

                // dbg!("Cleaning done. Items left: {}", map.len());
            }
        });

        AsyncCache { inner }
    }

    pub async fn set(&self, key: String, value: T, ttl_secs: u64) {
        let mut map = self.inner.lock().unwrap();

        let item = CacheItem::<T> {
            value,
            expires_at: Instant::now() + Duration::from_secs(ttl_secs),
        };

        map.insert(key, item);
    }

    pub async fn get(&self, key: &str) -> Option<T> {
        let map = self.inner.lock().unwrap();

        if let Some(item) = map.get(key) {
            if Instant::now() < item.expires_at {
                return Some(item.value.clone());
            }
        }
        None
    }
}
