use async_trait::async_trait;
use serde::Deserialize;

use crate::message::InternalMessage;

use super::Block;

#[derive(Debug, Deserialize)]
pub enum ConvertBodyConfig {
    Empty,
    JSON,
    OSC,
    Binary,
    String,
}

pub struct ConvertBodyBlock {
    pub config: ConvertBodyConfig,
}

#[async_trait]
impl Block for ConvertBodyBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        let data = match self.config {
            ConvertBodyConfig::Empty => message.data.to_empty(),
            ConvertBodyConfig::JSON => message.data.to_json(),
            ConvertBodyConfig::OSC => message.data.to_osc(),
            ConvertBodyConfig::Binary => message.data.to_binary(),
            ConvertBodyConfig::String => message.data.to_string(),
        };

        message.data = data?;

        Ok(vec![message])
    }
}
