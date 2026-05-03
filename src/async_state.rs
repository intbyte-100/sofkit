use gtk::glib;
use tokio::sync::{mpsc, oneshot};

use crate::state::{ReadState, WriteState};

const CHANNEL_CAPACITY: usize = 8;

#[derive(Clone)]
pub struct AsyncWriteState<T> {
    sender: mpsc::Sender<T>,
}

impl<T: 'static> AsyncWriteState<T> {
    pub fn new<S>(state: S) -> Self
    where
        S: WriteState<T> + 'static,
    {
        let (sender, mut receiver) = mpsc::channel::<T>(CHANNEL_CAPACITY);

        glib::spawn_future_local(async move {
            while let Some(value) = receiver.recv().await {
                state.replace(value);
            }
        });

        Self { sender }
    }

    pub async fn replace(&self, value: T) {
        let _ = self.sender.send(value).await;
    }
}

#[derive(Clone)]
pub struct AsyncReadState<T> {
    sender: mpsc::Sender<oneshot::Sender<T>>,
}

impl<T: Clone + 'static> AsyncReadState<T> {
    pub fn new<S>(state: S) -> Self
    where
        S: ReadState<T> + 'static,
    {
        let (sender, mut receiver) = mpsc::channel::<oneshot::Sender<T>>(CHANNEL_CAPACITY);

        glib::spawn_future_local(async move {
            while let Some(reply_tx) = receiver.recv().await {
                match state.get() {
                    Some(value) => {
                        let _ = reply_tx.send(value);
                    }
                    None => {
                        break;
                    }
                }
            }
        });

        Self { sender }
    }

    pub async fn snapshot(&self) -> Option<T> {
        let (reply_tx, reply_rx) = oneshot::channel::<T>();
        self.sender.send(reply_tx).await.ok()?;
        
        reply_rx.await.ok()
    }
}
