mod block;
mod cli;
mod connector;
mod message;
mod pipeline;

use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use cli::{Args, get_config};
use tokio::sync::mpsc;

use connector::{ConnectorHandle, make_connector};
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

    let (source_tx, mut source_rx) = mpsc::channel::<InternalMessage>(32);

    let pipeline = Arc::new(
        Pipeline::new(config.blocks, args.ignore_cycles).expect("Unable to create pipeline"),
    );

    println!("Starting connectors");

    let mut connector_handles: Vec<ConnectorHandle> = vec![];

    for (idx, con) in config.connectors.into_iter().enumerate() {
        println!("Starting connector with index {:#?}", idx);
        let handle = make_connector(idx, source_tx.clone(), con)
            .await
            .expect(&format!("Unable to make_connector with index {}", idx));
        connector_handles.push(handle);
    }

    let connector_handles = Arc::new(connector_handles);

    println!("Ready for incoming Messages");

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
