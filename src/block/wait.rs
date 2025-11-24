use std::time::Duration;

use async_trait::async_trait;

use crate::message::InternalMessage;

use super::Block;

pub struct WaitBlock {
    pub config: u64,
}

#[async_trait]
impl Block for WaitBlock {
    async fn exec(self: &Self, message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        tokio::time::sleep(Duration::from_millis(self.config)).await;

        Ok(vec![message])
    }
}
