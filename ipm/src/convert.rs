#[macro_use]
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
        }
    };
}
