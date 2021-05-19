use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::Received;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Trade {
    pub price: f32,
    pub volume: u64,
    pub figi: String,
    pub minute_rounded: DateTime<Utc>,
    pub sent: DateTime<Utc>,
    pub received: DateTime<Utc>,
}

impl Received for Trade {
    fn received(&self) -> DateTime<Utc> {
        self.received
    }
}
