use tokio::sync::broadcast::Receiver;

pub struct Shutdown {
    is_shutdown: bool,
    notify: Receiver<()>,
}

impl Shutdown {
    pub fn new(notify: Receiver<()>) -> Self {
        Self {
            notify,
            is_shutdown: false,
        }
    }

    pub(crate) fn is_shutdown(&self) -> bool {
        self.is_shutdown
    }

    pub(crate) async fn recv(&mut self) {
        if self.is_shutdown {
            return;
        }

        let _ = self.notify.recv().await;
        self.is_shutdown = true;
    }
}
