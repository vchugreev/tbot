use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const CANDLE_INTERVAL: &str = "1min";

#[derive(Serialize, Debug)]
pub struct SubscribeCandleReq {
    event: String,
    figi: String,
    interval: String,
}

impl SubscribeCandleReq {
    pub fn prepare(figi: String) -> SubscribeCandleReq {
        SubscribeCandleReq {
            event: "candle:subscribe".to_string(),
            figi,
            interval: CANDLE_INTERVAL.to_string(),
        }
    }
}

// https://tinkoffcreditsystems.github.io/invest-openapi/marketdata/#candlesubscribe
#[derive(Deserialize, Debug)]
#[serde(rename = "payload")]
pub struct CandlePayload {
    pub o: f32,
    pub c: f32,
    pub h: f32,
    pub l: f32,
    pub v: u64,
    pub interval: String,
    pub figi: String,
    pub time: DateTime<Utc>,
}
