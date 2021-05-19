macro_rules! order_book_from {
    ($p: path) => {
        type DomainOrderBook = crate::domain::order_book::OrderBook;
        paste::paste! {
            impl From<DomainOrderBook> for [<$p>]::OrderBook {
                fn from(item: DomainOrderBook) -> Self {
                    let bids: Vec<_> = item
                        .bids
                        .iter()
                        .map(|i| [<$p>]::OrderBookItem {
                            price: i.0,
                            volume: i.1,
                        })
                        .collect();

                    let asks: Vec<_> = item
                        .asks
                        .iter()
                        .map(|i| [<$p>]::OrderBookItem {
                            price: i.0,
                            volume: i.1,
                        })
                        .collect();

                    Self {
                        figi: item.figi,
                        depth: item.depth,
                        bids,
                        asks,
                        sent: item.sent.timestamp_millis(),
                        received: item.received.timestamp_millis(),
                    }
                }
            }

            impl From<[<$p>]::OrderBook> for DomainOrderBook {
                fn from(item: [<$p>]::OrderBook) -> Self {
                    use crate::server::services::proto::utils::convert_timestamp;
                    use crate::domain::order_book::Glass;

                    let bids: Glass = item
                        .bids
                        .iter()
                        .map(|i| (i.price, i.volume))
                        .collect();

                    let asks: Glass = item
                        .asks
                        .iter()
                        .map(|i| (i.price, i.volume))
                        .collect();

                    Self {
                        figi: item.figi,
                        depth: item.depth,
                        bids,
                        asks,
                        sent: convert_timestamp(item.sent),
                        received: convert_timestamp(item.received),
                    }
                }
            }
        }
    };
}

macro_rules! trade_from {
    ($p: path) => {
        type DomainTrade = crate::domain::trade::Trade;
        paste::paste! {
            impl From<DomainTrade> for [<$p>]::Trade {
                fn from(item: DomainTrade) -> Self {
                    Self {
                        price: item.price,
                        volume: item.volume,
                        figi: item.figi,
                        minute_rounded: item.minute_rounded.timestamp_millis(),
                        sent: item.sent.timestamp_millis(),
                        received: item.received.timestamp_millis(),
                    }
                }
            }

            impl From<[<$p>]::Trade> for DomainTrade {
                fn from(item: [<$p>]::Trade) -> Self {
                    use crate::server::services::proto::utils::convert_timestamp;
                    DomainTrade {
                        price: item.price,
                        volume: item.volume,
                        figi: item.figi,
                        minute_rounded: convert_timestamp(item.minute_rounded),
                        sent: convert_timestamp(item.sent),
                        received: convert_timestamp(item.received),
                    }
                }
            }
        }
    };
}
