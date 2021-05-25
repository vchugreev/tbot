use std::str::FromStr;

use log::{error, info};
use sqlx::migrate::Migrator;
use sqlx::pool::Pool;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{ConnectOptions, Postgres};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use super::queries::{add_order_book, add_trade, delete_today_order_books, delete_today_trades};
use crate::domain::order_book::OrderBook as DomainOrderBook;
use crate::domain::trade::Trade as DomainTrade;

const MIGRATIONS_DEFAULT_PATH: &str = "./migrations/";

pub async fn run(
    db_url: String,
    migrations_path: Option<&str>,
    trade_receiver: mpsc::Receiver<DomainTrade>,
    order_book_receiver: mpsc::Receiver<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let mut options = PgConnectOptions::from_str(db_url.as_str())?;
    options.disable_statement_logging(); // TODO: временное решение, ждем когда допилят настройку логирования https://github.com/launchbadge/sqlx/issues/942
    let pool = PgPoolOptions::new().max_connections(5).connect_with(options).await?;

    // Оставил в качестве примера, если делать без отключения логирования
    // let pool = PgPoolOptions::new().max_connections(5).connect(db_url.as_str()).await?;

    let path = match migrations_path {
        Some(p) => p,
        None => MIGRATIONS_DEFAULT_PATH,
    };

    let m = Migrator::new(std::path::Path::new(path)).await?;
    m.run(&pool).await?;

    // Запускаем очистку таблиц на сегодняшнюю дату - автоочистка удобна при многократном запуске
    delete_today_trades(&pool).await?;
    delete_today_order_books(&pool).await?;

    storing(pool, trade_receiver, order_book_receiver, shutdown).await;

    Ok(())
}

async fn storing(
    pool: Pool<Postgres>,
    mut trade_receiver: mpsc::Receiver<DomainTrade>,
    mut order_book_receiver: mpsc::Receiver<DomainOrderBook>,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                val = trade_receiver.recv() => {
                    if let Some(trade) = val {
                        if let Err(err) = add_trade(&pool, trade).await {
                            error!("insert trade failed: {:?}", err);
                        }
                    }
                },
                val = order_book_receiver.recv() => {
                    if let Some(order_book) = val {
                        if let Err(err) = add_order_book(&pool, order_book).await {
                            error!("insert order book failed: {:?}", err);
                        }
                    }
                },
                _ = shutdown.cancelled() => {
                    pool.close().await;
                    info!("storing finished");
                    return;
                }
            }
        }
    });
}
