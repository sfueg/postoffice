mod add_leading_slash;
mod convert_body;
mod lua_filter;
mod match_topic;
mod remove_body;
mod remove_leading_slash;
mod replace_body;
mod replace_topic;
mod wait;

use async_trait::async_trait;
use serde::Deserialize;

use add_leading_slash::AddLeadingSlashBlock;
use match_topic::{MatchTopicBlock, MatchTopicConfig};
use remove_body::RemoveBodyBlock;
use remove_leading_slash::RemoveLeadingSlashBlock;
use replace_body::ReplaceBodyBlock;
use replace_topic::ReplaceTopicBlock;

use crate::{
    block::{
        convert_body::{ConvertBodyBlock, ConvertBodyConfig},
        lua_filter::{LuaFilterBlock, LuaFilterConfig}, wait::WaitBlock,
    },
    message::InternalMessage,
};

#[derive(Debug, Deserialize)]
pub enum BlockConfig {
    AddLeadingSlash {
        to: Vec<Connection>,
    },
    RemoveLeadingSlash {
        to: Vec<Connection>,
    },
    RemoveBody {
        to: Vec<Connection>,
    },
    ReplaceBody {
        to: Vec<Connection>,
        config: serde_json::Value,
    },
    MatchTopic {
        to: Vec<Connection>,
        config: MatchTopicConfig,
    },
    ReplaceTopic {
        to: Vec<Connection>,
        config: String,
    },
    LuaFilter {
        to: Vec<Connection>,
        config: LuaFilterConfig,
    },
    ConvertBody {
        to: Vec<Connection>,
        config: ConvertBodyConfig,
    },
    Wait {
        to: Vec<Connection>,
        config: u64,
    },
}

pub struct BlockHandle {
    pub block: Box<dyn Block>,
    pub to: Vec<Connection>,
}

#[derive(Debug, Clone, Deserialize)]
pub enum Connection {
    Block(usize),
    Sink(usize),
}

pub fn make_block(config: BlockConfig) -> anyhow::Result<BlockHandle> {
    let (to, block): (Vec<Connection>, Box<dyn Block>) = match config {
        BlockConfig::AddLeadingSlash { to } => (to, Box::new(AddLeadingSlashBlock {})),
        BlockConfig::RemoveLeadingSlash { to } => (to, Box::new(RemoveLeadingSlashBlock {})),
        BlockConfig::RemoveBody { to } => (to, Box::new(RemoveBodyBlock {})),
        BlockConfig::ReplaceBody { to, config } => (to, Box::new(ReplaceBodyBlock { config })),
        BlockConfig::MatchTopic { to, config } => (to, Box::new(MatchTopicBlock { config })),
        BlockConfig::ReplaceTopic { to, config } => (to, Box::new(ReplaceTopicBlock { config })),
        BlockConfig::LuaFilter { to, config } => (to, Box::new(LuaFilterBlock::new(config)?)),
        BlockConfig::ConvertBody { to, config } => (to, Box::new(ConvertBodyBlock { config })),
        BlockConfig::Wait { to, config } => (to, Box::new(WaitBlock { config })),
    };

    return Ok(BlockHandle { block, to });
}

#[async_trait]
pub trait Block: Sync + Send {
    async fn exec(self: &Self, message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        Ok(vec![message])
    }
}
