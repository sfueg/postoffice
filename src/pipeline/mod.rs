use anyhow::Context;

use crate::{
    block::{BlockConfig, BlockHandle, Connection, make_block},
    message::InternalMessage,
};

pub struct Pipeline {
    blocks: Vec<BlockHandle>,
}

impl Pipeline {
    pub fn new(block_config: Vec<BlockConfig>, ignore_cycles: bool) -> anyhow::Result<Self> {
        let mut blocks = vec![];
        for config in block_config {
            blocks.push(make_block(config)?);
        }

        let pipeline = Self { blocks };

        let cycles = pipeline.get_cycles();

        if cycles.len() > 0 {
            let path = cycles
                .iter()
                .map(|path| {
                    path.iter()
                        .map(|entry| format!("Block {}", entry))
                        .collect::<Vec<_>>()
                        .join(" -> ")
                })
                .collect::<Vec<_>>()
                .join("\n");

            if ignore_cycles {
                println!(
                    "
[WARN]: Detected cycles in config:
{}
Ignoring cycles because of --ignore-cycles
",
                    path
                );
            } else {
                panic!(
                    "
[WARN]: Detected cycles in config:
{}
Not all invariants are covered - the config may still be valid
Run with --ignore-cycles to ignore cycles
",
                    path
                );
            }
        }

        return Ok(pipeline);
    }

    pub async fn handle_message_with_connections(
        self: &Self,
        to: &Vec<Connection>,
        message: InternalMessage,
        collector: &mut Vec<(usize, InternalMessage)>,
    ) -> anyhow::Result<()> {
        for to in to {
            match to {
                Connection::Block(idx) => {
                    Box::pin(self.handle_message(*idx, message.clone(), collector)).await?;
                }
                Connection::Sink(idx) => {
                    collector.push((*idx, message.clone()));
                }
            }
        }

        return Ok(());
    }

    pub async fn handle_message(
        self: &Self,
        block_idx: usize,
        message: InternalMessage,
        collector: &mut Vec<(usize, InternalMessage)>,
    ) -> anyhow::Result<()> {
        let handle = self
            .blocks
            .get(block_idx)
            .context(format!("Missing block with index {}", block_idx))?;

        let next_messages = handle.block.exec(message).await?;

        if next_messages.len() == 0 {
            println!("Block {:#?} dropped all messages", block_idx);
        } else if next_messages.len() > 1 {
            println!(
                "Block {:#?} created {:#?} new messages",
                block_idx,
                next_messages.len() - 1
            );
        }

        for message in next_messages {
            self.handle_message_with_connections(&handle.to, message, collector).await?;
        }

        return Ok(());
    }

    fn get_cycles(self: &Self) -> Vec<Vec<usize>> {
        return self
            .blocks
            .iter()
            .enumerate()
            .filter_map(|(block_idx, _)| self.get_cycles_from_block(block_idx))
            .collect();
    }

    fn get_cycles_from_block(self: &Self, block_idx: usize) -> Option<Vec<usize>> {
        let mut path = vec![block_idx];

        let block = self.blocks.get(block_idx).unwrap();

        for to in &block.to {
            match to {
                Connection::Block(block_idx) => {
                    if self.traverse_to_check_cycles(&mut path, *block_idx) {
                        return Some(path);
                    }
                }
                Connection::Sink(_) => {}
            }
        }

        return None;
    }

    fn traverse_to_check_cycles(self: &Self, path: &mut Vec<usize>, block_idx: usize) -> bool {
        if path.contains(&block_idx) {
            // The block was already visited -> cycle
            return true;
        } else {
            // Add self
            path.push(block_idx);

            let block = self.blocks.get(block_idx).unwrap();
            for to in &block.to {
                match to {
                    Connection::Block(block_idx) => {
                        if self.traverse_to_check_cycles(path, *block_idx) {
                            return true;
                        }
                    }
                    Connection::Sink(_) => {}
                }
            }

            // Remove self
            path.pop();

            return false;
        }
    }
}
