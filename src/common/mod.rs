use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// Retrieve the current unix time in nanoseconds
pub fn get_unix_time_millis() -> u128 {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_else(|_| {
        println!("Failed computing UNIX timestamp during admin login!");
        Duration::from_secs(0)
    });
    duration.as_millis()
}
/// Retrieve the current unix time in nanoseconds
pub fn get_unix_time_seconds() -> u64 {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap_or_else(|_| {
        println!("Failed computing UNIX timestamp during admin login!");
        Duration::from_secs(0)
    });
    duration.as_secs()
}

#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("No matches found")]
    NotFound,
    #[error("Could not get from database")]
    Get,
    #[error("Could not set to database")]
    Set,
    #[error("Could not communicate with database")]
    Communicate,
    #[error("Could not deserialize binary data")]
    Deserialize,
    #[error("Could not serialize binary data")]
    Serialize,
    #[error("Could not delete from database")]
    NoDelete,
}
