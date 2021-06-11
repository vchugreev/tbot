use chrono::NaiveDate;
use flexi_logger::Logger;
use log::info;
use tokio::sync::{broadcast, mpsc};
use tokio::time;
use tokio_util::sync::CancellationToken;

use args::{Args, Mode};
use domain::{order_book::OrderBook, trade::Trade};
use server::receiver::ReceiverMaker;
use settings::Settings;

mod args;
mod db;
mod domain;
mod server;
mod settings;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::new();
    let mode = args.get_mode()?;

    let cfg: Settings = Settings::new(args.get_configs_path()).expect("configs can't be loaded");

    Logger::with_str(cfg.log.level.as_str())
        .format(flexi_logger::colored_detailed_format)
        .start()?;

    info!("price repository started");

    let shutdown = run_ctrlc()?;

    // через этот канал будем транслировать данные наружу, т.е. из базы вовне, fec - for external consumers
    let (fec_trade_sender, _) = broadcast::channel::<Trade>(20);
    let (fec_order_book_sender, _) = broadcast::channel::<OrderBook>(20);

    // через этот канал будем транслировать данные внутри системы, т.е. в базу, fdc - for db consumer
    let (fdc_trade_sender, fdc_trade_receiver) = mpsc::channel::<Trade>(20);
    let (fdc_order_book_sender, fdc_order_book_receiver) = mpsc::channel::<OrderBook>(20);

    server::run(
        cfg.server.addr,
        ReceiverMaker::<Trade>::new(fec_trade_sender.clone()),
        ReceiverMaker::<OrderBook>::new(fec_order_book_sender.clone()),
        fdc_trade_sender,
        fdc_order_book_sender,
        shutdown.clone(),
    )
    .await?;

    match mode {
        Mode::Storing => {
            db::storing::run(
                cfg.db.url.clone(),
                args.get_migrations_path(),
                fdc_trade_receiver,
                fdc_order_book_receiver,
                shutdown.clone(),
            )
            .await?;
        }
        Mode::Reading { date, speed } => {
            let date = NaiveDate::parse_from_str(&date, "%Y-%m-%d").unwrap();
            db::reading::run(
                cfg.db.url.clone(),
                date,
                speed,
                fec_trade_sender,
                fec_order_book_sender,
                shutdown.clone(),
            )
            .await?;
        }
    }

    // До этого были неблокирующие вызовы, поэтому ждем сигнала о завершении и блокируем поток
    shutdown.cancelled().await;

    // Нужно дать время другием фоновым задачам завершить свои дела
    time::sleep(time::Duration::from_secs(1)).await;
    info!("service finished");

    Ok(())
}

fn run_ctrlc() -> anyhow::Result<CancellationToken> {
    let token = CancellationToken::new();
    let t = token.clone();
    ctrlc::set_handler(move || {
        let _ = t.cancel();
    })?;

    Ok(token)
}
