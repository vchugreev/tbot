use std::convert::From;
use std::pin::Pin;

use futures::Stream;
use log::{debug, error, info};
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use incoming::price_stream_server::{PriceStream, PriceStreamServer};
use incoming::{Empty, OrderBook, Trade};

use crate::receiver::ReceiverMaker;

pub mod incoming {
    tonic::include_proto!("incoming");
}

trade_from!(incoming);
order_book_from!(incoming);

pub struct PriceStreamService {
    trade_rm: ReceiverMaker<DomainTrade>,
    order_book_rm: ReceiverMaker<DomainOrderBook>,
}

impl PriceStreamService {
    pub fn new(trade_rm: ReceiverMaker<DomainTrade>, order_book_rm: ReceiverMaker<DomainOrderBook>) -> Self {
        PriceStreamService {
            trade_rm,
            order_book_rm,
        }
    }
}

#[tonic::async_trait]
impl PriceStream for PriceStreamService {
    type SubscribeToTradeStream = Pin<Box<dyn Stream<Item = Result<Trade, Status>> + Send + Sync + 'static>>;

    async fn subscribe_to_trade(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::SubscribeToTradeStream>, Status> {
        let mut receiver = self.trade_rm.receiver();
        let output = async_stream::try_stream! {
            loop {
                match receiver.recv().await {
                    Ok(trade) =>  {
                        debug!("grpc input: {:?}", trade);
                        yield Trade::from(trade);
                    },
                    Err(err) => {
                        error!("receiving data failed: {}", err)
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::SubscribeToTradeStream))
    }

    type SubscribeToOrderBookStream = Pin<Box<dyn Stream<Item = Result<OrderBook, Status>> + Send + Sync + 'static>>;

    async fn subscribe_to_order_book(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::SubscribeToOrderBookStream>, Status> {
        let mut receiver = self.order_book_rm.receiver();
        let output = async_stream::try_stream! {
            loop {
                match receiver.recv().await {
                    Ok(order_book) => {
                        debug!("grpc input: {:?}", order_book.clone());
                        yield OrderBook::from(order_book);
                    },
                    Err(err) => {
                        error!("receiving data failed: {}", err)
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::SubscribeToOrderBookStream))
    }
}

pub async fn run(
    addr: String,
    trade_rm: ReceiverMaker<DomainTrade>,
    order_book_rm: ReceiverMaker<DomainOrderBook>,
    shutdown: CancellationToken,
) -> anyhow::Result<()> {
    let addr = addr.parse()?;
    info!("price stream server listening on: {}", addr);

    let service = PriceStreamService::new(trade_rm, order_book_rm);
    let svc = PriceStreamServer::new(service);

    tokio::spawn(async move {
        let res = Server::builder()
            .add_service(svc)
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
