use std::{net::SocketAddrV4, str::FromStr};

use rosc::{OscMessage, OscPacket};
use serde::Deserialize;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{block::Connection, message::InternalMessage};

use super::{ConnectorHandle, SourceTX};

#[derive(Debug, Deserialize)]
pub struct OSCSendConnectorConfig {
    pub host: String,
    pub port: u16,
}

pub async fn make_osc_send_connector(
    _idx: usize,
    _source_tx: SourceTX,
    config: OSCSendConnectorConfig,
    to: Option<Vec<Connection>>,
) -> anyhow::Result<ConnectorHandle> {
    let (sink_tx, mut sink_rx) = mpsc::channel::<InternalMessage>(32);

    let host_addr = SocketAddrV4::from_str("0.0.0.0:0")?;
    let to_addr = SocketAddrV4::from_str(format!("{}:{}", config.host, config.port).as_str())?;
    let sock = UdpSocket::bind(host_addr).await?;

    tokio::task::spawn(async move {
        loop {
            if let Some(msg) = sink_rx.recv().await {
                if let Ok(args) = msg.data.get_osc() {
                    let msg_buf = rosc::encoder::encode(&OscPacket::Message(OscMessage {
                        addr: msg.topic,
                        args,
                    }));

                    if let Ok(msg_buf) = msg_buf {
                        sock.send_to(&msg_buf, to_addr).await.ok();
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
