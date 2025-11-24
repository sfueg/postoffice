use async_trait::async_trait;

use crate::message::InternalMessage;

use super::Block;

pub struct RemoveLeadingSlashBlock {}

#[async_trait]
impl Block for RemoveLeadingSlashBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        if message.topic.starts_with("/") {
            message.topic = message.topic.trim_start_matches("/").to_string();
        }

        Ok(vec![message])
    }
}