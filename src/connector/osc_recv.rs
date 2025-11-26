use std::{net::SocketAddrV4, str::FromStr};

use rosc::OscPacket;
use serde::Deserialize;
use tokio::{net::UdpSocket, sync::mpsc};

use crate::{
    block::Connection,
    message::{InternalMessage, InternalMessageData},
};

use super::{ConnectorHandle, SourceTX};

#[derive(Debug, Deserialize)]
pub struct OSCRecvConnectorConfig {
    pub interface: String,
    pub port: u16,
}

pub async fn make_osc_recv_connector(
    idx: usize,
    source_tx: SourceTX,
    config: OSCRecvConnectorConfig,
    to: Option<Vec<Connection>>,
) -> anyhow::Result<ConnectorHandle> {
    let (sink_tx, mut _sink_rx) = mpsc::channel::<InternalMessage>(32);

    let addr = format!("{}:{}", config.interface, config.port);
    let addr = SocketAddrV4::from_str(addr.as_str())?;
    let sock = UdpSocket::bind(addr).await?;

    tokio::task::spawn(async move {
        let mut buf = [0u8; rosc::decoder::MTU];

        loop {
            if let Ok(size) = sock.recv(&mut buf).await {
                if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                    let messages = collect_messages_from_osc_packet(idx, packet);

                    for message in messages {
                        source_tx
                            .send(message)
                            .await
                            .expect("Unable to send messages to source_rx");
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

fn collect_messages_from_osc_packet(source_idx: usize, packet: OscPacket) -> Vec<InternalMessage> {
    match packet {
        OscPacket::Message(osc_message) => vec![InternalMessage {
            source_connector_idx: source_idx,
            topic: osc_message.addr,
            data: InternalMessageData::OSC(osc_message.args),
        }],
        OscPacket::Bundle(osc_bundle) => osc_bundle
            .content
            .into_iter()
            .map(|packet| collect_messages_from_osc_packet(source_idx, packet))
            .flatten()
            .collect(),
    }
}
