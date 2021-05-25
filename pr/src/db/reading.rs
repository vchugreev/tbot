use std::fmt::Debug;
use std::ops::Add;
use std::str::FromStr;

use anyhow::Context;
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};
use futures::TryStreamExt;
use log::{debug, error, info};
use serde::Deserialize;
use sqlx::pool::PoolConnection;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Postgres, Row};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;

use super::queries::select_min_received_by_date;
use crate::domain::common::Received;
use crate::domain::order_book::OrderBook as DomainOrderBook;
use crate::domain::trade::Trade as DomainTrade;

const TRADE_TABLE: &str = "trade";
const ORDER_BOOK_TABLE: &str = "order_book";

pub async fn run(
    db_url: String,
    date: NaiveDate,
    speed: u16,
    trade_sender: broadcast::Sender<DomainTrade>,
    order_book_sender: broadcast::Sender<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let mut options = PgConnectOptions::from_str(db_url.as_str())?;
    options.disable_statement_logging();
    let pool = PgPoolOptions::new().max_connections(5).connect_with(options).await?;

    let dt = select_min_received_by_date(&pool, date)
        .await
        .context("query to select min received failed")?
        .ok_or(anyhow::anyhow!("minimum datetime not founded"))?;

    let dt_start = DateTime::<Utc>::from_utc(dt, Utc);
    info!("datetime started: {}", dt_start);

    let conn = pool.acquire().await.unwrap();
    let trade_task = reading::<DomainTrade>(
        conn,
        TRADE_TABLE.to_string(),
        date,
        dt_start,
        speed,
        trade_sender,
        shutdown.clone(),
    )
    .await;

    let conn = pool.acquire().await.unwrap();
    let order_book_task = reading::<DomainOrderBook>(
        conn,
        ORDER_BOOK_TABLE.to_string(),
        date,
        dt_start,
        speed,
        order_book_sender,
        shutdown.clone(),
    )
    .await;

    let _ = tokio::join!(trade_task, order_book_task);

    Ok(())
}

async fn pause(prev: DateTime<Utc>, cur: DateTime<Utc>, speed: u16) {
    let mut millis = (cur - prev).num_milliseconds() as u64;
    millis /= speed as u64;
    if millis > 0 {
        tokio::time::sleep(tokio::time::Duration::from_millis(millis)).await;
    }
}

/// Обобщенное решение для чтения Trade и OrderBook
async fn reading<T: 'static + Received + Send + for<'de> Deserialize<'de> + Debug>(
    mut conn: PoolConnection<Postgres>,
    table: String,
    date: NaiveDate,
    dt_start: DateTime<Utc>,
    speed: u16,
    sender: broadcast::Sender<T>,
    shutdown: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let (begin, end) = get_interval(date);
        let mut dt_prev = dt_start;

        let query = format!(
            "SELECT content FROM {} WHERE received >= $1 AND received < $2 ORDER BY id",
            table
        );

        let mut stream = sqlx::query(query.as_str()).bind(begin).bind(end).fetch(&mut conn);

        loop {
            tokio::select! {
                val = stream.try_next() => {
                    match val {
                        Ok(opt_row) => {
                            match opt_row {
                                Some(row) => {
                                    let content = row.try_get("content").unwrap();
                                    let item: T = serde_json::from_value(content).unwrap();

                                    debug!("trade: {:?}", item);

                                    let received = item.received();
                                    pause(dt_prev, received, speed).await;
                                    dt_prev = received;

                                    if sender.receiver_count() > 0 {
                                        if let Err(err) = sender.send(item) {
                                            error!("send item failed, error: {}", err);
                                        }
                                    }
                                },
                                None => {
                                    // все, данные закончились, выходим (это нормальный сценарий завершения)
                                    info!("streaming finished");
                                    return;
                                }
                            }
                        },
                        Err(err) => {
                            error!("unexpected error: {}", err);
                            return;
                        }
                    }
                },
                _ = shutdown.cancelled() => {
                    info!("streaming finished");
                    return;
                }
            }
        }
    })
}

pub(super) fn get_interval(date: NaiveDate) -> (NaiveDateTime, NaiveDateTime) {
    let begin = date.and_hms(0, 0, 0);
    let end = begin.add(Duration::days(1));
    (begin, end)
}
