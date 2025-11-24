use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;

use crate::message::InternalMessage;

use super::Block;

#[derive(Debug, Deserialize)]
pub enum MatchTopicConfig {
    Exact(String),
    StartsWith(String),
    EndsWith(String),
    Regex(String),
}

pub struct MatchTopicBlock {
    pub config: MatchTopicConfig,
    pub regex: Option<Regex>,
}

impl MatchTopicBlock {
    pub fn new(config: MatchTopicConfig) -> anyhow::Result<MatchTopicBlock> {
        let regex = match config {
            MatchTopicConfig::Regex(ref pattern) => Some(Regex::new(pattern)?),
            _ => None,
        };

        Ok(MatchTopicBlock { config, regex })
    }
}

#[async_trait]
impl Block for MatchTopicBlock {
    async fn exec(self: &Self, message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        let is_match: bool = match &self.config {
            MatchTopicConfig::Exact(pattern) => *pattern == message.topic,
            MatchTopicConfig::StartsWith(pattern) => message.topic.starts_with(pattern),
            MatchTopicConfig::EndsWith(pattern) => message.topic.ends_with(pattern),
            MatchTopicConfig::Regex(_) => {
                if let Some(ref regex) = self.regex {
                    regex.is_match(&message.topic)
                } else {
                    // This should never happen
                    return Err(anyhow::anyhow!("Missing Regex Pattern"));
                }
            }
        };

        if is_match {
            Ok(vec![message])
        } else {
            Ok(vec![])
        }
    }
}
