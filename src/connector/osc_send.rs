use std::{net::SocketAddrV4, str::FromStr};

use rosc::{OscMessage, OscPacket};
use serde::Deserialize;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{message::InternalMessage, block::Connection};

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

    tokio::task::spawn(async move {
        let host_addr = SocketAddrV4::from_str("0.0.0.0:0").unwrap();
        let to_addr =
            SocketAddrV4::from_str(format!("{}:{}", config.host, config.port).as_str()).unwrap();
        let sock = UdpSocket::bind(host_addr).await.unwrap();

        loop {
            if let Some(msg) = sink_rx.recv().await {
                let msg_buf = rosc::encoder::encode(&OscPacket::Message(OscMessage {
                    addr: msg.topic,
                    args: msg.data.get_osc().unwrap(),
                }))
                .unwrap();

                sock.send_to(&msg_buf, to_addr).await.unwrap();
            }
        }
    });

    return Ok(ConnectorHandle {
        sink_tx,
        to: to.unwrap_or_default(),
    });
}
