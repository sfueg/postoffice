use async_trait::async_trait;
use serde::Deserialize;

use crate::message::InternalMessage;

use super::Block;

#[derive(Debug, Deserialize)]
pub enum MatchTopicConfig {
    Exact(String),
    StartsWith(String),
    EndsWith(String),
    Pattern(String),
}

pub struct MatchTopicBlock {
    pub config: MatchTopicConfig,
}

#[async_trait]
impl Block for MatchTopicBlock {
    async fn exec(self: &Self, message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        let matches: bool = match &self.config {
            MatchTopicConfig::Exact(pattern) => *pattern == message.topic,
            MatchTopicConfig::StartsWith(pattern) => message.topic.starts_with(pattern),
            MatchTopicConfig::EndsWith(pattern) => message.topic.ends_with(pattern),
            MatchTopicConfig::Pattern(pattern) => todo!(),
        };

        if matches {
            Ok(vec![message])
        } else {
            Ok(vec![])
        }
    }
}
