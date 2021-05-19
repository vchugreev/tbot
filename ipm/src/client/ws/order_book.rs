use serde::{Deserialize, Serialize};

const ORDER_BOOK_DEPTH: u32 = 10;

#[derive(Serialize, Debug)]
pub struct SubscribeOrderBookReq {
    event: String,
    figi: String,
    depth: u32,
}

impl SubscribeOrderBookReq {
    pub fn prepare(figi: String) -> SubscribeOrderBookReq {
        SubscribeOrderBookReq {
            event: "orderbook:subscribe".to_string(),
            figi,
            depth: ORDER_BOOK_DEPTH,
        }
    }
}

// https://tinkoffcreditsystems.github.io/invest-openapi/marketdata/#orderbooksubscribe
#[derive(Deserialize, Debug)]
#[serde(rename = "payload")]
pub struct OrderBookPayload {
    pub figi: String,
    pub depth: u32,
    pub bids: Vec<(f32, u64)>,
    pub asks: Vec<(f32, u64)>,
}
