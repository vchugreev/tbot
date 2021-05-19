use std::pin::Pin;

use futures::Stream;
use log::{debug, error};
use tonic::{Request, Response, Status};

use incoming::price_stream_server::PriceStream;
use incoming::{Empty, OrderBook, Trade};

use crate::server::receiver::ReceiverMaker;

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
                        error!("receiving trade failed: {}", err)
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
                        error!("receiving order book failed: {}", err)
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::SubscribeToOrderBookStream))
    }
}
