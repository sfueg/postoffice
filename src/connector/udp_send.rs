use std::{net::SocketAddrV4, str::FromStr};

use serde::Deserialize;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{
    block::Connection,
    lifecycle::{LifeCycleMessage, LifeCycleTX},
    message::InternalMessage,
};

use super::{ConnectorHandle, SourceTX};

#[derive(Debug, Deserialize)]
pub struct UDPSendConnectorConfig {
    pub host: String,
    pub port: u16,
}

pub async fn make_udp_send_connector(
    idx: usize,
    _source_tx: SourceTX,
    config: UDPSendConnectorConfig,
    to: Option<Vec<Connection>>,
    lifecycle_tx: LifeCycleTX,
) -> anyhow::Result<ConnectorHandle> {
    let (sink_tx, mut sink_rx) = mpsc::channel::<InternalMessage>(32);

    let host_addr = SocketAddrV4::from_str("0.0.0.0:0")?;
    let to_addr = SocketAddrV4::from_str(format!("{}:{}", config.host, config.port).as_str())?;
    let sock = UdpSocket::bind(host_addr).await?;

    tokio::task::spawn(async move {
        lifecycle_tx
            .send(LifeCycleMessage::Ready { idx })
            .await
            .expect("Failed to send LifeCycleMessage");

        loop {
            if let Some(msg) = sink_rx.recv().await {
                let topic = msg.topic;

                match sock.send_to(&topic.as_bytes(), to_addr).await {
                    Ok(_) => {}
                    Err(e) => {
                        lifecycle_tx
                            .send(LifeCycleMessage::Failed { idx, err: e.into() })
                            .await
                            .expect("Failed to send LifeCycleMessage");
                    }
                }
            }
        }
    });

    return Ok(ConnectorHandle {
        sink_tx,
        to: to.unwrap_or_default(),
    });
}
