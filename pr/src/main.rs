use anyhow::Context;
use chrono::NaiveDate;
use clap::{App, Arg, ArgMatches};
use flexi_logger::Logger;
use log::info;
use tokio::sync::{broadcast, mpsc};
use tokio::time;
use tokio_util::sync::CancellationToken;

use settings::Settings;

use self::domain::{order_book::OrderBook, trade::Trade};
use self::server::receiver::ReceiverMaker;

mod db {
    pub(super) mod queries;
    pub mod reading;
    pub mod storing;
}

mod server {
    pub mod grpc;
    pub mod receiver;
    #[macro_use]
    pub mod services {
        #[macro_use]
        pub mod proto {
            #[macro_use]
            pub mod convert;
            pub mod utils;
        }
        pub mod price_storage;
        pub mod price_stream;
    }
}

mod settings;

mod domain {
    pub mod common;
    pub mod order_book;
    pub mod trade;
}

const CONFIGS: &str = "configs";
const MIGRATIONS: &str = "migrations";
const STORING: &str = "storing";
const READING: &str = "reading";
const SPEED: &str = "speed";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = get_args();

    let configs_path = args.value_of(CONFIGS).unwrap_or("");
    let migrations_path = args.value_of(MIGRATIONS).unwrap_or("");

    let is_storing = args.is_present(STORING);

    // В случае режима чтения мы указываем не только флаг, но и дату, для которой запускаем этот режим.
    let reading_date = args.value_of(READING).unwrap_or("");
    let speed = get_speed(&args)?;

    // Обязательно должен быть указан один из двух режимов, в котором запущен сервис
    if !is_storing && reading_date.is_empty() {
        let err = anyhow::Error::msg("startup mode not defined, must be specified -s or -r (storing or reading)");
        return anyhow::Result::Err(err);
    }

    if is_storing && !reading_date.is_empty() {
        // Недостижимость этого условия обеспечивает conflicts_with в get_args
        unreachable!("it must by only one mode: storing or reading");
    }

    // Одновременно использовать оба режима не допускается
    // Эта логика реализована в get_args через .conflicts_with

    let cfg: Settings = Settings::new(configs_path).expect("configs can't be loaded");

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

    let trade_rm = ReceiverMaker::<Trade>::new(fec_trade_sender.clone());
    let order_book_rm = ReceiverMaker::<OrderBook>::new(fec_order_book_sender.clone());
    server::grpc::run(
        cfg.server.addr,
        trade_rm,
        order_book_rm,
        fdc_trade_sender,
        fdc_order_book_sender,
        shutdown.clone(),
    )
    .await?;

    if is_storing {
        db::storing::run(
            cfg.db.url.clone(),
            migrations_path,
            fdc_trade_receiver,
            fdc_order_book_receiver,
            shutdown.clone(),
        )
        .await?;
    }

    if !reading_date.is_empty() {
        let date = NaiveDate::parse_from_str(reading_date, "%Y-%m-%d").unwrap();
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

    // До этого были неблокирующие вызовы, поэтому ждем сигнала о завершении и блокируем поток
    shutdown.cancelled().await;

    // Нужно дать время другием фоновым задачам завершить свои дела
    time::sleep(time::Duration::from_secs(1)).await;
    info!("service finished");

    Ok(())
}

fn get_speed(args: &ArgMatches) -> anyhow::Result<u16> {
    let speed = args.value_of(SPEED).unwrap_or("1");
    let result = speed
        .parse::<u16>()
        .context("speed must by only unsigned integer: 1, 2, 3, ...")?;
    if result == 0 {
        let e = anyhow::Error::msg("speed must be greater than zero");
        return Err(e);
    }
    Ok(result)
}

fn get_args() -> ArgMatches {
    App::new("price repository")
        .version("0.1.0")
        .about("tinkoff investments microservice for storage")
        .arg(
            Arg::new(CONFIGS)
                .short('c')
                .long(CONFIGS)
                .value_name("PATH TO CONFIGS")
                .about("sets a custom path to configuration files"),
        )
        .arg(
            Arg::new(MIGRATIONS)
                .short('m')
                .long(MIGRATIONS)
                .value_name("PATH TO MIGRATIONS")
                .about("sets a custom path to migrations files"),
        )
        .arg(
            Arg::new(STORING)
                .short('s')
                .long(STORING)
                .value_name("STORING MODE")
                .takes_value(false)
                .conflicts_with_all(&[READING, SPEED])
                .about("sets a storing mode"),
        )
        .arg(
            Arg::new(READING)
                .short('r')
                .long(READING)
                .value_name("READING MODE")
                .conflicts_with(STORING)
                .requires(SPEED)
                .about("sets a reading mode"),
        )
        .arg(
            Arg::new(SPEED)
                .index(1)
                .default_value("1")
                .about("sets a speed rate reading"),
        )
        .get_matches()
}

fn run_ctrlc() -> anyhow::Result<CancellationToken> {
    let token = CancellationToken::new();
    let t = token.clone();
    ctrlc::set_handler(move || {
        let _ = t.cancel();
    })?;

    Ok(token)
}
