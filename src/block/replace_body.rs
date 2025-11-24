use async_trait::async_trait;

use crate::message::{InternalMessage, InternalMessageData};

use super::Block;

pub struct ReplaceBodyBlock {
    pub config: serde_json::Value,
}

#[async_trait]
impl Block for ReplaceBodyBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        message.data = InternalMessageData::Json(self.config.clone());

        Ok(vec![message])
    }
}