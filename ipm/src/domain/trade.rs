use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct Trade {
    pub price: f32,
    pub volume: u64,
    pub figi: String,
    pub minute_rounded: DateTime<Utc>,
    pub sent: DateTime<Utc>,
    pub received: DateTime<Utc>,
}

impl Trade {
    pub fn new(
        price: f32,
        volume: u64,
        figi: String,
        minute_rounded: DateTime<Utc>,
        sent: DateTime<Utc>,
        received: DateTime<Utc>,
    ) -> Self {
        Trade {
            price,
            volume,
            figi,
            minute_rounded,
            sent,
            received,
        }
    }
}
