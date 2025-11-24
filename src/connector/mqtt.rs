use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    block::Connection,
    message::{InternalMessage, InternalMessageData},
};

use super::{ConnectorHandle, SourceTX};

#[derive(Debug, Deserialize)]
pub struct MQTTConnectorConfig {
    pub client_id: Option<String>,
    pub host: String,
    pub port: u16,
    pub topics: Option<Vec<String>>,
}

pub async fn make_mqtt_connector(
    idx: usize,
    source_tx: SourceTX,
    config: MQTTConnectorConfig,
    to: Option<Vec<Connection>>,
) -> anyhow::Result<ConnectorHandle> {
    let (sink_tx, mut sink_rx) = mpsc::channel::<InternalMessage>(32);

    let is_source = match to {
        Some(ref connections) => connections.len() > 0,
        None => false,
    };

    tokio::task::spawn(async move {
        let mut mqttoptions = MqttOptions::new(
            config.client_id.unwrap_or("postoffice".to_string()),
            config.host,
            config.port,
        );

        mqttoptions.set_keep_alive(Duration::from_secs(5));

        let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

        let c2 = client.clone();
        tokio::task::spawn(async move {
            loop {
                let msg = sink_rx.recv().await;

                match msg {
                    Some(msg) => {
                        c2.publish(
                            msg.topic,
                            QoS::AtLeastOnce,
                            false,
                            msg.data.get_binary().unwrap(),
                        )
                        .await
                        .unwrap();
                    }
                    None => todo!(),
                }
            }
        });

        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(packet)) => match packet {
                    rumqttc::Packet::ConnAck(_) => {
                        subscribe_to_topics(&client, &config.topics, is_source)
                            .await
                            .unwrap();
                    }
                    rumqttc::Packet::Publish(publish) => {
                        let msg = InternalMessage {
                            source_connector_idx: idx,
                            topic: publish.topic,
                            data: InternalMessageData::Binary(publish.payload),
                        };

                        source_tx.send(msg).await.unwrap();
                    }
                    _ => {}
                },
                Ok(_) => {}
                Err(e) => {
                    println!("[MQTT]: ConnectionError: {:#?}", e);
                }
            }
        }
    });

    return Ok(ConnectorHandle {
        sink_tx,
        to: to.unwrap_or_default(),
    });
}

async fn subscribe_to_topics(
    client: &AsyncClient,
    topics: &Option<Vec<String>>,
    is_source: bool,
) -> anyhow::Result<()> {
    if is_source {
        if let Some(topics) = topics {
            for topic in topics {
                client.subscribe(topic, QoS::ExactlyOnce).await.unwrap();
            }
        } else {
            client.subscribe("#", QoS::ExactlyOnce).await.unwrap();
        }
    } else {
        println!("[MQTT]: Skipping subscribe because it will drop all messages anyway...");
    }

    return Ok(());
}
