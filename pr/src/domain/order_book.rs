use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::common::Received;

pub type Glass = Vec<(f32, u64)>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OrderBook {
    pub figi: String,
    pub depth: u32,
    pub bids: Glass,
    pub asks: Glass,
    pub sent: DateTime<Utc>,
    pub received: DateTime<Utc>,
}

impl Received for OrderBook {
    fn received(&self) -> DateTime<Utc> {
        self.received
    }
}
