use chrono::{DateTime, Utc};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::Deserialize;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_util::sync::CancellationToken;
use tungstenite::{handshake::client::Request, Message};

use crate::domain::{order_book::OrderBook, trade::Trade};
use crate::settings::Tinkoff;

use super::candle::{CandlePayload, SubscribeCandleReq};
use super::order_book::{OrderBookPayload, SubscribeOrderBookReq};

pub async fn run(
    cfg: Tinkoff,
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let stream = prepare_web_socket(cfg).await?;
    reading(stream, trade_sender, order_book_sender, shutdown).await
}

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

async fn prepare_web_socket(cfg: Tinkoff) -> anyhow::Result<WebSocket> {
    let request = Request::builder()
        .method("GET")
        .uri(cfg.ws.clone())
        .header("Authorization", format!("Bearer {}", cfg.token.clone()))
        .body(())?;

    let (mut stream, response) = connect_async(request).await.expect("failed to ws connect");

    info!("connected to the server, response HTTP code: {}", response.status());

    for figi in cfg.figis.iter() {
        stream.send(prepare_subscribe_candle_req(figi)).await?;
        stream.send(prepare_subscribe_order_book_req(figi)).await?;
    }

    anyhow::Result::Ok(stream)
}

fn extract_message(msg: Message) -> Option<StreamMsg> {
    let res = msg.to_text();
    if res.is_err() {
        return None;
    }

    let text = res.unwrap();
    if text.is_empty() {
        return None;
    }

    let message = serde_json::from_str(text) as serde_json::Result<StreamMsg>;
    if message.is_err() {
        error!("unknown json format: {}", msg.to_text().unwrap());
        return None;
    }

    Some(message.unwrap())
}

async fn reading(
    mut stream: WebSocket,
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let result = tokio::spawn(async move {
        loop {
            let result: Option<anyhow::Result<()>> = tokio::select! {
                val = stream.next() => {
                    match val {
                        Some(item) => process_new_item(item, trade_sender.clone(), order_book_sender.clone()),
                        None => None,
                    }
                },
                _ = shutdown.cancelled() => {
                    info!("ws client finished");
                    Some(anyhow::Result::Ok(()))
                }
            };

            if let Some(res) = result {
                return res;
            }
        }
    })
    .await
    .unwrap();

    result
}

fn process_new_item(
    item: Result<Message, tungstenite::error::Error>,
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
) -> Option<anyhow::Result<()>> {
    if item.is_err() {
        // Возможные ошибки: https://docs.rs/tungstenite/0.13.0/tungstenite/error/enum.Error.html
        let err = item.err().unwrap();
        log::error!("streaming error: {:?}", err);
        let e = anyhow::Error::msg(format!("streaming error: {:?}", err));
        return Some(anyhow::Result::Err(e)); // Выходим с ошибкой, это приведет к переподключению (start_and_restart_ws_client)
    }

    // Возможные типы сообщения: https://docs.rs/tungstenite/0.13.0/tungstenite/enum.Message.html
    let message = item.unwrap();

    if message.is_close() {
        log::warn!("stream closed");
        let e = anyhow::Error::msg("stream closed");
        return Some(anyhow::Result::Err(e)); // Выходим с ошибкой, это приведет к переподключению (start_and_restart_ws_client)
    }

    if !message.is_text() {
        return None; // Какой-то другой тип сообщения, тиа Ping, нам это не интересно
    }

    if let Some(msg) = extract_message(message) {
        match msg {
            StreamMsg::Candle { payload, time } => {
                debug!("payload: {:?}, time: {}, now: {}", payload, time, Utc::now());

                if trade_sender.receiver_count() == 0 {
                    return None; // Отправлять некому, поэтому досрочный выход
                }

                let trade = Trade::new(payload.c, payload.v, payload.figi, payload.time, time, Utc::now());

                debug!(
                    "it is difference between sent and received: {} (ms)",
                    trade.received.signed_duration_since(trade.sent).num_milliseconds()
                );

                if let Err(err) = trade_sender.send(trade) {
                    error!("send trade failed, error: {}", err);
                }
            }

            StreamMsg::OrderBook { payload, time } => {
                debug!("payload: {:?}, time: {}", payload, time);

                if order_book_sender.receiver_count() == 0 {
                    return None; // Отправлять некому, поэтому досрочный выход
                }

                let order_book = OrderBook::new(
                    payload.figi,
                    payload.depth,
                    payload.bids,
                    payload.asks,
                    time,
                    Utc::now(),
                );

                debug!(
                    "it is difference between sent and received: {} (ms)",
                    order_book
                        .received
                        .signed_duration_since(order_book.sent)
                        .num_milliseconds()
                );

                if let Err(err) = order_book_sender.send(order_book) {
                    error!("send order book failed, error: {}", err);
                }
            }
        }
    }

    None
}

fn prepare_subscribe_candle_req(figi: &str) -> Message {
    let req = SubscribeCandleReq::prepare(figi.to_string());
    let msg = serde_json::to_string(&req).unwrap();
    Message::Text(msg)
}

fn prepare_subscribe_order_book_req(figi: &str) -> Message {
    let req = SubscribeOrderBookReq::prepare(figi.to_string());
    let msg = serde_json::to_string(&req).unwrap();
    Message::Text(msg)
}

// Это то, что мы читаем по ws с Tinkoff https://tinkoffcreditsystems.github.io/invest-openapi/marketdata/
#[derive(Deserialize, Debug)]
#[serde(tag = "event")]
enum StreamMsg {
    #[serde(rename = "candle")]
    Candle {
        payload: CandlePayload,
        time: DateTime<Utc>,
    },
    #[serde(rename = "orderbook")]
    OrderBook {
        payload: OrderBookPayload,
        time: DateTime<Utc>,
    },
}
