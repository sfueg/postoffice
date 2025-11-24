use async_trait::async_trait;

use crate::message::InternalMessage;

use super::Block;

pub struct RemoveBodyBlock {}

#[async_trait]
impl Block for RemoveBodyBlock {
    async fn exec(self: &Self, mut message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        message.data = message.data.to_empty()?;

        Ok(vec![message])
    }
}
