use async_trait::async_trait;

use crate::message::InternalMessage;

use super::Block;

pub struct AddLeadingSlashBlock {}

#[async_trait]
impl Block for AddLeadingSlashBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        if !message.topic.starts_with("/") {
            message.topic = format!("/{}", message.topic)
        }

        Ok(vec![message])
    }
}
