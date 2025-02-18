use anyhow::Result;
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};

use crate::config::RedisCacheConfig;

#[derive(Clone)]
pub struct RedisCache {
    // Connections are cheap to clone.
    conn: MultiplexedConnection,
    config: RedisCacheConfig,
}

impl RedisCache {
    pub async fn connect(config: &RedisCacheConfig) -> Result<Self> {
        let client: Client = Client::open(config.address.clone())?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            conn,
            config: config.clone(),
        })
    }

    pub async fn hset(&self, key: &str, field: &str, value: &str) -> Result<()> {
        let key = if let Some(prefix) = self.config.key_prefix.as_ref() {
            format!("{prefix}:{key}")
        } else {
            key.to_string()
        };

        let mut conn = self.conn.clone();
        let _: i32 = conn.hset(key, field, value).await?;

        Ok(())
    }
}
