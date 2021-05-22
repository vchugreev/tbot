use log::{debug, error};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};

use incoming::{OrderBook, Trade};
use storage::price_storage_server::PriceStorage;
use storage::Resp;

// В storage есть заимствованные структуры (message) из incoming (Trade и OrderBook), поэтому incoming тоже нужно подключать
pub mod incoming {
    tonic::include_proto!("incoming");
}

pub mod storage {
    tonic::include_proto!("storage");
}

trade_from!(incoming);
order_book_from!(incoming);

#[derive(Debug)]
pub struct PriceStorageService {
    trade_sender: mpsc::Sender<DomainTrade>,
    order_book_sender: mpsc::Sender<DomainOrderBook>,
}

impl PriceStorageService {
    pub fn new(trade_sender: mpsc::Sender<DomainTrade>, order_book_sender: mpsc::Sender<DomainOrderBook>) -> Self {
        PriceStorageService {
            trade_sender,
            order_book_sender,
        }
    }
}

#[tonic::async_trait]
impl PriceStorage for PriceStorageService {
    async fn add_trade(&self, request: Request<Trade>) -> Result<Response<Resp>, Status> {
        debug!("request: {:?}", request);

        let trade = request.into_inner();
        debug!("extracted request: {:?}", trade);

        let t = DomainTrade::from(trade);

        // Отправляем через try_send для того, чтобы отработать ситуацию переполнения канала.
        // Если принимающая сторона - db application service не успевает обработать поступающие данные,
        // канал переполниться и мы увидим это в логах.
        if let Err(err) = self.trade_sender.try_send(t) {
            error!("send trade failed, error: {}", err);
        }

        Ok(Response::new(Resp::default()))
    }

    async fn add_order_book(&self, request: Request<OrderBook>) -> Result<Response<Resp>, Status> {
        debug!("request: {:?}", request);

        let order_book = request.into_inner();
        debug!("extracted request: {:?}", order_book);

        let ob = DomainOrderBook::from(order_book);
        if let Err(err) = self.order_book_sender.try_send(ob) {
            error!("send order book failed, error: {}", err);
        }

        Ok(Response::new(Resp::default()))
    }
}
