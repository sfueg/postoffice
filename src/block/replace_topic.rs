use async_trait::async_trait;

use crate::message::InternalMessage;

use super::Block;

pub struct ReplaceTopicBlock {
    pub config: String,
}

#[async_trait]
impl Block for ReplaceTopicBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        message.topic = self.config.clone();

        Ok(vec![message])
    }
}
