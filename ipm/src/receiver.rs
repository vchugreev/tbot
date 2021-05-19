use tokio::sync::broadcast;

pub struct ReceiverMaker<T: Clone>(broadcast::Sender<T>);

impl<T: Clone> ReceiverMaker<T> {
    pub fn new(sender: broadcast::Sender<T>) -> Self {
        Self(sender)
    }

    pub fn receiver(&self) -> broadcast::Receiver<T> {
        self.0.subscribe()
    }
}
