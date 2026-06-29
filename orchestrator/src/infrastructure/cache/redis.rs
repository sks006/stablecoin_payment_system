use redis::AsyncCommands;

pub struct RedisCache {
    client: redis::Client,
}

impl RedisCache {
    pub fn new(redis_url: &str) -> Result<Self, crate::domain::error::Error> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| crate::domain::error::Error::Cache(e.to_string()))?;
        Ok(Self { client })
    }

    pub async fn get(&self, key: &str) -> Result<Option<String>, crate::domain::error::Error> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| crate::domain::error::Error::Cache(e.to_string()))?;
        
        let val: Option<String> = conn.get(key)
            .await
            .map_err(|e| crate::domain::error::Error::Cache(e.to_string()))?;
            
        Ok(val)
    }

    pub async fn set(&self, key: &str, value: &str, ttl_secs: usize) -> Result<(), crate::domain::error::Error> {
        let mut conn = self.client.get_async_connection()
            .await
            .map_err(|e| crate::domain::error::Error::Cache(e.to_string()))?;
            
        conn.set_ex(key, value, ttl_secs)
            .await
            .map_err(|e| crate::domain::error::Error::Cache(e.to_string()))?;
            
        Ok(())
    }
}
