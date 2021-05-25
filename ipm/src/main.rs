use clap::{App, Arg, ArgMatches};
use flexi_logger::Logger;
use log::{error, info};
use tokio::sync::broadcast;
use tokio::time;
use tokio_util::sync::CancellationToken;

use receiver::ReceiverMaker;

use self::client::ws::{emulator as ws_emulator, reader as ws_reader};
use self::domain::{order_book::OrderBook, trade::Trade};
use self::settings::{Settings, Tinkoff};

#[macro_use]
mod proto;
mod client;
mod domain;
mod receiver;
mod server;
mod settings;

const CONFIGS: &str = "configs";
const WS_EMULATE: &str = "ws_emulate";
const REPOSITORY: &str = "repository";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = get_args();
    let configs_path = args.value_of(CONFIGS).unwrap_or("");
    let ws_emulate = args.is_present(WS_EMULATE);
    let repository = args.is_present(REPOSITORY);

    let cfg: Settings = Settings::new(configs_path).expect("configs can't be loaded");

    Logger::with_str(cfg.log.level.as_str())
        .format(flexi_logger::colored_detailed_format)
        .start()?;

    info!("incoming price manager started");

    let shutdown = run_ctrlc()?;
    let (trade_sender, _) = broadcast::channel::<Trade>(20);
    let (order_book_sender, _) = broadcast::channel::<OrderBook>(20);

    if ws_emulate {
        ws_emulator::run(
            &cfg.client.tinkoff.figis,
            trade_sender.clone(),
            order_book_sender.clone(),
        )
        .await?;
    } else {
        start_and_restart_ws_client(
            cfg.client.tinkoff,
            trade_sender.clone(),
            order_book_sender.clone(),
            shutdown.clone(),
        )
        .await;
    }

    let (trade_rm, order_book_rm) = create_receivers(trade_sender.clone(), order_book_sender.clone());
    server::grpc::run(cfg.server.addr, trade_rm, order_book_rm, shutdown.clone()).await?;

    if repository {
        let (trade_rm, order_book_rm) = create_receivers(trade_sender.clone(), order_book_sender.clone());
        start_and_restart_grpc_client(cfg.client.pr.addr, trade_rm, order_book_rm, shutdown.clone()).await;
    }

    // До этого были неблокирующие вызовы, поэтому ждем сигнала о завершении и блокируем поток
    shutdown.cancelled().await;

    // Нужно дать время другим фоновым задачам завершить свои дела
    time::sleep(time::Duration::from_secs(1)).await;
    info!("service finished");

    Ok(())
}

/// Небольшая вспомогательная функция - хелпер, сделал только для того, чтобы не загромождать основной код
fn create_receivers(
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
) -> (ReceiverMaker<Trade>, ReceiverMaker<OrderBook>) {
    (ReceiverMaker::new(trade_sender), ReceiverMaker::new(order_book_sender))
}

/// Запуск ws клиента и его и перезапуск в случае потери соединения
async fn start_and_restart_ws_client(
    tinkoff: Tinkoff,
    trade_sender: broadcast::Sender<Trade>,
    order_book_sender: broadcast::Sender<OrderBook>,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Err(err) = ws_reader::run(tinkoff.clone(), trade_sender.clone(), order_book_sender.clone(), shutdown.clone()) => {
                    error!("ws client not running: {}, retry will be in 1 second", err);
                    time::sleep(time::Duration::from_secs(1)).await;
                },
                _ = shutdown.cancelled() => {
                    return;
                }
            }
        }
    });
}

async fn start_and_restart_grpc_client(
    addr: String,
    trade_rm: ReceiverMaker<Trade>,
    order_book_rm: ReceiverMaker<OrderBook>,
    shutdown: CancellationToken,
) {
    tokio::spawn(async move {
        loop {
            tokio::select! {
                Err(err) = client::grpc::run(addr.clone(), trade_rm.receiver(), order_book_rm.receiver(), shutdown.clone()) => {
                    error!("grpc client not running: {}, retry will be in 1 second", err);
                    time::sleep(time::Duration::from_secs(1)).await;
                },
                _ = shutdown.cancelled() => {
                    return;
                }
            }
        }
    });
}

fn get_args() -> ArgMatches {
    App::new("incoming price manager")
        .version("0.1.0")
        .about("tinkoff investments microservice for price stream reading")
        .arg(
            Arg::new(CONFIGS)
                .short('c')
                .long(CONFIGS)
                .value_name("PATH TO CONFIGS")
                .about("sets a custom path to configuration files"),
        )
        .arg(
            Arg::new(WS_EMULATE)
                .short('e')
                .long(WS_EMULATE)
                .value_name("WS EMULATE")
                .takes_value(false)
                .about("sets a ws emulate mode"),
        )
        .arg(
            Arg::new(REPOSITORY)
                .short('r')
                .long(REPOSITORY)
                .value_name("REPOSITORY SENDING")
                .takes_value(false)
                .about("sets a to repository sending mode"),
        )
        .get_matches()
}

/// Ждем нажатия ctr-c, в случае нажатия через CancellationToken инициируем распространение сигнала на заверешение
/// По сути CancellationToken - это широковещательный канал
fn run_ctrlc() -> anyhow::Result<CancellationToken> {
    let token = CancellationToken::new();
    let t = token.clone();
    ctrlc::set_handler(move || {
        let _ = t.cancel();
    })?;
    Ok(token)
}
