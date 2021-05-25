pub mod grpc;
pub mod ws {
    pub(self) mod candle;
    pub mod emulator;
    pub(self) mod order_book;
    pub mod reader;
}
