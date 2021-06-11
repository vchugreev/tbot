use log::{error, info};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;

use crate::domain::order_book::OrderBook as DomainOrderBook;
use crate::domain::trade::Trade as DomainTrade;

use receiver::ReceiverMaker;
use services::{
    price_storage::{storage::price_storage_server::PriceStorageServer, PriceStorageService},
    price_stream::{incoming::price_stream_server::PriceStreamServer, PriceStreamService},
};

pub mod receiver;
#[macro_use]
pub mod services;

/// Запускает два сервиса в рамках одного grpc сервера.
/// Один сервис транслирует данные из базы наружу, а другой - извне в базу данных.
/// Напрямую к базе они не обращаются, вся работа с базой вынесена в отдельный application service (db),
/// взаимодействие с db организовано через каналы:
/// trade_rm - генерация trade ресиверов для получения данных транслируемых наружу сервиса,
/// order_book_rm - генерация order book ресиверов для получения данных транслируемых наружу сервиса,
/// fdc_trade_sender (fdc - for db consumer) - сендер для отправки trade в базу,
/// fdc_order_book_sender (fdc - for db consumer) - сендер для отправки order book в базу.
pub async fn run(
    addr: String,
    trade_rm: ReceiverMaker<DomainTrade>,
    order_book_rm: ReceiverMaker<DomainOrderBook>,
    fdc_trade_sender: mpsc::Sender<DomainTrade>,
    fdc_order_book_sender: mpsc::Sender<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let addr = addr.parse()?;
    info!("grpc server listening on: {}", addr);

    // этот сервис транслирует поток данных, вычитанных из базы наружу, это нужно, чтобы воспроизводить исторические данные
    let stream_service = PriceStreamService::new(trade_rm, order_book_rm);

    // этот сервис обрабатывает вызовы, которые инициируют добавление данных в базу, т.е. через его эндпоинты можно добавить данные в базу
    let storage_service = PriceStorageService::new(fdc_trade_sender, fdc_order_book_sender);

    tokio::spawn(async move {
        let res = Server::builder()
            .add_service(PriceStreamServer::new(stream_service))
            .add_service(PriceStorageServer::new(storage_service))
            .serve_with_shutdown(addr, async {
                shutdown.cancelled().await;
                info!("grpc server finished");
            })
            .await;

        if res.is_err() {
            error!("could not start the grpc server");
        }
    });

    Ok(())
}
