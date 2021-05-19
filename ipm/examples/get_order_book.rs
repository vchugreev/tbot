use std::error::Error;

use chrono::Utc;
use flexi_logger::Logger;
use log::info;
use tonic::transport::Channel;
use tonic::Request;

use incoming::price_stream_client::PriceStreamClient;
use incoming::Empty;

pub mod incoming {
    tonic::include_proto!("incoming");
}

const URL: &str = "https://[::1]:10000";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Logger::with_str("info")
        .format(flexi_logger::colored_detailed_format)
        .start()?;

    let mut client = PriceStreamClient::connect(URL).await?;
    info!("grpc client started");
    get_order_book(&mut client).await?;

    Ok(())
}

async fn get_order_book(client: &mut PriceStreamClient<Channel>) -> Result<(), Box<dyn Error>> {
    let mut stream = client
        .subscribe_to_order_book(Request::new(Empty {}))
        .await?
        .into_inner();

    while let Some(msg) = stream.message().await? {
        let now = Utc::now().timestamp_millis();
        let diff = now - msg.received; // "транспортные расходы" - время затраченное на передачу данных от ipm сюда

        info!("message: {:?}, shipping costs (ms): {}", msg, diff);
    }

    Ok(())
}
