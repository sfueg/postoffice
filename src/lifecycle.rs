use std::collections::HashMap;

use tokio::sync::mpsc;

pub struct LifeCycleHandler {
    all_ready_rx: mpsc::Receiver<()>,
    pub lifecycle_tx: LifeCycleTX,
}

impl LifeCycleHandler {
    pub fn start(connector_idxs: Vec<usize>) -> Self {
        let (tx, mut rx) = mpsc::channel::<LifeCycleMessage>(32);
        let (all_ready_tx, all_ready_rx) = mpsc::channel::<()>(32);

        tokio::spawn(async move {
            let mut m = HashMap::<usize, bool>::new();

            for idx in connector_idxs {
                m.insert(idx, false);
            }

            loop {
                let msg = rx.recv().await.expect("Unable to recv from LifecycleRX");

                match msg {
                    LifeCycleMessage::Ready { idx } => {
                        println!("[Connector {}] Ready", idx);

                        m.insert(idx, true);

                        if m.iter().all(|(_, state)| *state) {
                            all_ready_tx
                                .send(())
                                .await
                                .expect("Unable to send all_ready signal");
                        }
                    }
                    LifeCycleMessage::Disconnected { idx, err } => {
                        println!("[Connector {}] Disconnected because of: {}", idx, err);
                    }
                    LifeCycleMessage::Failed { idx, err } => {
                        println!("[Connector {}] Failed because of: {}", idx, err);
                    }
                    LifeCycleMessage::Exited { idx, err } => {
                        println!("[Connector {}] Exited because of: {}", idx, err);
                        std::process::exit(1);
                    }
                }
            }
        });

        Self {
            all_ready_rx,
            lifecycle_tx: tx,
        }
    }

    pub async fn wait_all_ready(mut self: Self) -> anyhow::Result<()> {
        if self.all_ready_rx.recv().await.is_none() {
            panic!("Unable to receive all_ready signal")
        }

        Ok(())
    }
}

pub enum LifeCycleMessage {
    Ready { idx: usize },
    Disconnected { idx: usize, err: anyhow::Error },
    Failed { idx: usize, err: anyhow::Error },
    Exited { idx: usize, err: anyhow::Error },
}

pub type LifeCycleTX = mpsc::Sender<LifeCycleMessage>;
