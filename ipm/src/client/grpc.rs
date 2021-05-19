use log::{error, info};
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tonic::transport::Channel;
use tonic::Request;

use incoming::{OrderBook, Trade};
use repository::price_storage_client::PriceStorageClient;

// В repository есть заимствованные структуры (message) из incoming (Trade и OrderBook), поэтому incoming тоже нужно подключать
pub mod incoming {
    tonic::include_proto!("incoming");
}

pub mod repository {
    tonic::include_proto!("storage");
}

trade_from!(incoming);
order_book_from!(incoming);

pub async fn run(
    addr: String,
    trade_receiver: broadcast::Receiver<DomainTrade>,
    order_book_receiver: broadcast::Receiver<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let url = format!("https://{}", addr);
    let client = PriceStorageClient::connect(url).await?;
    info!("price repository connected");
    sending(client, trade_receiver, order_book_receiver, shutdown).await
}

async fn sending(
    mut client: PriceStorageClient<Channel>,
    mut trade_receiver: broadcast::Receiver<DomainTrade>,
    mut order_book_receiver: broadcast::Receiver<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let result = tokio::spawn(async move {
        loop {
            let result: Option<anyhow::Result<()>> = tokio::select! {
                val = trade_receiver.recv() => {
                    match val {
                        Ok(trade) => {
                            send_trade(&mut client, trade).await
                        },
                        Err(err) => {
                            error!("unexpected error: {:?}", err);
                            None
                        }
                    }
                },
                val = order_book_receiver.recv() => {
                    match val {
                        Ok(order_book) => {
                            send_order_book(&mut client, order_book).await
                        },
                        Err(err) => {
                            error!("unexpected error: {:?}", err);
                            None
                        }
                    }
                },
                _ = shutdown.cancelled() => {
                    info!("grpc client finished");
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

async fn send_trade(client: &mut PriceStorageClient<Channel>, trade: DomainTrade) -> Option<anyhow::Result<()>> {
    let t = Trade::from(trade);
    let request = Request::new(t);
    let response = client.add_trade(request).await;
    match response {
        Ok(_) => None,
        Err(err) => {
            error!("trade sending error: {:?}", err);
            let e = anyhow::Error::msg(format!("streaming error: {:?}", err)); // Конвертируем tonic::Status в anyhow::Error
            Some(anyhow::Result::Err(e)) // Выходим с ошибкой, это приведет к переподключению (start_and_restart_grpc_client)
        }
    }
}

async fn send_order_book(
    client: &mut PriceStorageClient<Channel>,
    order_book: DomainOrderBook,
) -> Option<anyhow::Result<()>> {
    let ob = OrderBook::from(order_book);
    let request = tonic::Request::new(ob);
    let response = client.add_order_book(request).await;
    match response {
        Ok(_) => None,
        Err(err) => {
            error!("order book sending error: {:?}", err);
            let e = anyhow::Error::msg(format!("streaming error: {:?}", err));
            Some(anyhow::Result::Err(e))
        }
    }
}
