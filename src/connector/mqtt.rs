use std::time::Duration;

use rumqttc::{AsyncClient, MqttOptions, QoS};
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::{
    block::Connection,
    lifecycle::{LifeCycleMessage, LifeCycleTX},
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
    lifecycle_tx: LifeCycleTX,
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
        let lifecycle_tx2 = lifecycle_tx.clone();
        tokio::task::spawn(async move {
            loop {
                let msg = sink_rx.recv().await;

                match msg {
                    Some(msg) => {
                        if let Ok(payload) = msg.data.get_binary() {
                            match c2
                                .publish(msg.topic, QoS::AtLeastOnce, false, payload)
                                .await
                            {
                                Ok(_) => {}
                                Err(e) => {
                                    lifecycle_tx2
                                        .clone()
                                        .send(LifeCycleMessage::Failed { idx, err: e.into() })
                                        .await
                                        .expect("Failed to send LifeCycleMessage");
                                }
                            }
                        }
                    }
                    None => panic!(
                        "Unable to recv from sink_rx. This means that all sink_tx are closed"
                    ),
                }
            }
        });

        loop {
            match eventloop.poll().await {
                Ok(rumqttc::Event::Incoming(packet)) => match packet {
                    rumqttc::Packet::ConnAck(_) => {
                        subscribe_to_topics(&client, &config.topics, is_source)
                            .await
                            .expect("Unable to subscribe to topics");

                        lifecycle_tx
                            .send(LifeCycleMessage::Ready { idx })
                            .await
                            .expect("Failed to send LifeCycleMessage");
                    }
                    rumqttc::Packet::Publish(publish) => {
                        let msg = InternalMessage {
                            source_connector_idx: idx,
                            topic: publish.topic,
                            data: InternalMessageData::Binary(publish.payload),
                        };

                        source_tx
                            .send(msg)
                            .await
                            .expect("Unable to send messages to source_rx");
                    }
                    _ => {}
                },
                Ok(_) => {}
                Err(e) => lifecycle_tx
                    .send(LifeCycleMessage::Disconnected { idx, err: e.into() })
                    .await
                    .expect("Failed to send LifeCycleMessage"),
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
                client.subscribe(topic, QoS::ExactlyOnce).await?;
            }
        } else {
            client.subscribe("#", QoS::ExactlyOnce).await?;
        }
    } else {
        println!("[MQTT]: Skipping subscribe because it will drop all messages anyway...");
    }

    return Ok(());
}
