mod block;
mod cli;
mod connector;
mod lifecycle;
mod message;
mod pipeline;

use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use cli::{Args, get_config};
use tokio::sync::mpsc;

use connector::{ConnectorHandle, make_connector};
use lifecycle::{LifeCycleHandler, LifeCycleMessage};
use message::InternalMessage;
use pipeline::Pipeline;

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let config =
        get_config(&args).expect(format!("Can't read config at \"{}\"", args.file).as_str());

    if args.debug {
        println!("{:#?}", config);
    }

    let life_cycle_handler = LifeCycleHandler::start(
        config
            .connectors
            .iter()
            .enumerate()
            .map(|(idx, _)| idx)
            .collect(),
    );

    let (source_tx, mut source_rx) = mpsc::channel::<InternalMessage>(32);

    let pipeline = Arc::new(
        Pipeline::new(config.blocks, args.ignore_cycles).expect("Unable to create pipeline"),
    );

    println!("Starting connectors");

    let mut connector_handles: Vec<ConnectorHandle> = vec![];

    for (idx, con) in config.connectors.into_iter().enumerate() {
        println!("[Connector {}] Starting", idx);
        let handle = make_connector(
            idx,
            source_tx.clone(),
            con,
            life_cycle_handler.lifecycle_tx.clone(),
        )
        .await;

        match handle {
            Ok(handle) => {
                connector_handles.push(handle);
            }
            Err(e) => {
                life_cycle_handler
                    .lifecycle_tx
                    .send(LifeCycleMessage::Exited { idx, err: e.into() })
                    .await
                    .expect("Failed to send LifeCycleMessage");
            }
        }
    }

    let connector_handles = Arc::new(connector_handles);

    println!("Waiting for connectors");

    life_cycle_handler
        .wait_all_ready()
        .await
        .expect("Unable to wait_all_ready");

    println!("Startup complete");

    loop {
        let incoming = source_rx.recv().await;

        match incoming {
            Some(incoming) => {
                if args.debug {
                    println!("\nIncoming Message {:#?}", incoming);
                } else {
                    println!("\nIncoming Message with Topic {:#?}", incoming.topic);
                }

                let pipeline = pipeline.clone();
                let connector_handles = connector_handles.clone();

                tokio::spawn(async move {
                    let handle = connector_handles
                        .get(incoming.source_connector_idx)
                        .context(format!(
                            "Missing connector with index {}",
                            incoming.source_connector_idx,
                        ))
                        .expect("Failed to get connector")
                        .clone();

                    let mut collector: Vec<(usize, InternalMessage)> = vec![];

                    pipeline
                        .handle_message_with_connections(&handle.to, incoming, &mut collector)
                        .await
                        .expect("Failed to handle message");

                    if args.debug {
                        println!("Collected messages {:#?}", collector);
                    } else {
                        println!("Collected {:#?} messages", collector.len());
                    }

                    for (sink_idx, message) in collector {
                        let handle = connector_handles
                            .get(sink_idx)
                            .context(format!("Missing sink with index {}", sink_idx))
                            .expect("Failed to get sink");

                        handle.sink_tx.send(message).await.expect(&format!(
                            "Failed to send message to sink_rx with idx {}",
                            sink_idx
                        ));
                    }
                });
            }
            None => {
                // This can only happen if `source_rx.recv()` returns `None` which means
                // that all `source_tx` channel halfs are closed.
                panic!(
                    "source_rx.recv() returned None which means that no work can be done at this point"
                )
            }
        }
    }
}
