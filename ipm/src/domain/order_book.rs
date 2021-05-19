use chrono::{DateTime, Utc};

pub type Glass = Vec<(f32, u64)>;

#[derive(Clone, Debug)]
pub struct OrderBook {
    pub figi: String,
    pub depth: u32,
    pub bids: Glass,
    pub asks: Glass,
    pub sent: DateTime<Utc>,
    pub received: DateTime<Utc>,
}

impl OrderBook {
    pub fn new(
        figi: String,
        depth: u32,
        bids: Glass,
        asks: Glass,
        sent: DateTime<Utc>,
        received: DateTime<Utc>,
    ) -> Self {
        OrderBook {
            figi,
            depth,
            bids,
            asks,
            sent,
            received,
        }
    }
}
