use chrono::{DurationRound, Utc};
use futures::future::{join_all, FutureExt};
use log::error;
use rand::Rng;
use tokio::sync::broadcast;

use crate::domain::{order_book::OrderBook, trade::Trade};

pub async fn run(
    figis: &[String],
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
) -> anyhow::Result<()> {
    let mut futures = Vec::new();

    for figi in figis.iter() {
        futures.push(emulate_trade(figi.clone(), trade_sender.clone()).boxed());
        futures.push(emulate_order_book(figi.clone(), order_book_sender.clone()).boxed());
    }

    join_all(futures).await;

    Ok(())
}

async fn emulate_trade(figi: String, sender: broadcast::Sender<Trade>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let trade = Trade::new(
                1.0,
                1,
                figi.clone(),
                now.duration_round(chrono::Duration::minutes(1)).unwrap(),
                now,
                now,
            );

            if sender.receiver_count() > 0 {
                if let Err(err) = sender.send(trade) {
                    error!("send trade failed, error: {}", err);
                }
            }

            let n = rand::thread_rng().gen_range(1000..2000);
            tokio::time::sleep(tokio::time::Duration::from_millis(n)).await;
        }
    })
}

async fn emulate_order_book(figi: String, sender: broadcast::Sender<OrderBook>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let now = Utc::now();
            let order_book = OrderBook::new(figi.clone(), 1, vec![(1.0, 1)], vec![(1.0, 1)], now, now);

            if sender.receiver_count() > 0 {
                if let Err(err) = sender.send(order_book) {
                    error!("send trade failed, error: {}", err);
                }
            }

            let n = rand::thread_rng().gen_range(500..1000);
            tokio::time::sleep(tokio::time::Duration::from_millis(n)).await;
        }
    })
}
