pub mod mqtt;
pub mod osc_recv;
pub mod osc_send;
pub mod udp_send;

use mqtt::{MQTTConnectorConfig, make_mqtt_connector};
use osc_recv::{OSCRecvConnectorConfig, make_osc_recv_connector};
use osc_send::{OSCSendConnectorConfig, make_osc_send_connector};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{block::Connection, connector::udp_send::{make_udp_send_connector, UDPSendConnectorConfig}, message::InternalMessage};

#[derive(Debug, Clone)]
pub struct ConnectorHandle {
    pub to: Vec<Connection>,
    pub sink_tx: mpsc::Sender<InternalMessage>,
}

pub type SourceTX = mpsc::Sender<InternalMessage>;

#[derive(Debug, Deserialize)]
pub enum ConnectorConfig {
    MQTT {
        config: MQTTConnectorConfig,
        to: Option<Vec<Connection>>,
    },
    OSCRecv {
        config: OSCRecvConnectorConfig,
        to: Option<Vec<Connection>>,
    },
    OSCSend {
        config: OSCSendConnectorConfig,
    },
    UDPSend {
        config: UDPSendConnectorConfig,
    },
    // TODO: HTTPRecvServer
    // TODO: HTTPRecvSSE
    // TODO: HTTPSendClient
    // TODO: HTTPSendSSE
}

pub async fn make_connector(
    idx: usize,
    source_tx: SourceTX,
    config: ConnectorConfig,
) -> anyhow::Result<ConnectorHandle> {
    match config {
        ConnectorConfig::MQTT { config, to } => {
            make_mqtt_connector(idx, source_tx, config, to).await
        }
        ConnectorConfig::OSCRecv { config, to } => {
            make_osc_recv_connector(idx, source_tx, config, to).await
        }
        ConnectorConfig::OSCSend { config } => {
            make_osc_send_connector(idx, source_tx, config, None).await
        }
        ConnectorConfig::UDPSend { config } => {
            make_udp_send_connector(idx, source_tx, config, None).await
        }
    }
}
