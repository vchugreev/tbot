use chrono::{Duration, DurationRound, NaiveDate, NaiveDateTime, Utc};
use sqlx::{pool::Pool, types::Json, Postgres};

use crate::domain::order_book::OrderBook as DomainOrderBook;
use crate::domain::trade::Trade as DomainTrade;

use super::reading::get_interval;

pub async fn add_trade(pool: &Pool<Postgres>, trade: DomainTrade) -> anyhow::Result<i32> {
    let rec = sqlx::query!(
        r#"
INSERT
INTO trade (figi, received, content)
VALUES ($1, $2, $3)
RETURNING id
        "#,
        trade.figi.clone(),
        trade.received.naive_utc(),
        Json(trade) as _
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

pub async fn add_order_book(pool: &Pool<Postgres>, order_book: DomainOrderBook) -> anyhow::Result<i32> {
    let rec = sqlx::query!(
        r#"
INSERT
INTO order_book (figi, received, content)
VALUES ($1, $2, $3)
RETURNING id
        "#,
        order_book.figi.clone(),
        order_book.received.naive_utc(),
        Json(order_book) as _
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.id)
}

pub async fn delete_today_trades(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let today = Utc::now().duration_trunc(Duration::days(1)).unwrap().naive_utc();
    sqlx::query!(
        r#"
DELETE
FROM trade
WHERE received >= $1
        "#,
        today
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_today_order_books(pool: &Pool<Postgres>) -> anyhow::Result<()> {
    let today = Utc::now().duration_trunc(Duration::days(1)).unwrap().naive_utc();
    sqlx::query!(
        r#"
DELETE
FROM order_book
WHERE received >= $1
        "#,
        today
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn select_min_received_by_date(
    pool: &Pool<Postgres>,
    date: NaiveDate,
) -> anyhow::Result<Option<NaiveDateTime>> {
    let (begin, end) = get_interval(date);
    let rec = sqlx::query!(
        r#"
SELECT min(mc)
FROM (
    SELECT min(received) AS mc FROM trade WHERE received >= $1 AND received < $2
    UNION
    SELECT min(received) AS mc FROM order_book WHERE received >= $1 AND received < $2
) AS received
        "#,
        begin,
        end
    )
    .fetch_one(pool)
    .await?;

    Ok(rec.min)
}
