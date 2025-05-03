use std::sync::{Mutex, OnceLock};
use snowflake::SnowflakeIdBucket;

static SNOWFLAKE_BUCKET: OnceLock<Mutex<SnowflakeIdBucket>> = OnceLock::new();

pub struct SnowflakeUtil;

impl SnowflakeUtil {
    pub fn get_id() -> Result<u64, Box<dyn std::error::Error>> {
        let bucket = SNOWFLAKE_BUCKET.get_or_init(|| {
            let machine_id = std::env::var("SNOWFLAKE_MACHINE_ID")
                .unwrap_or_else(|_| "0".into())
                .parse::<i32>()
                .expect("Invalid machine ID");

            let node_id = std::env::var("SNOWFLAKE_NODE_ID")
                .unwrap_or_else(|_| "0".into())
                .parse::<i32>()
                .expect("Invalid node ID");

            Mutex::new(SnowflakeIdBucket::new(machine_id, node_id))
        });

        let mut guard = bucket.lock().map_err(|_| "Mutex poison error")?;
        Ok(guard.get_id() as u64)
    }
} 