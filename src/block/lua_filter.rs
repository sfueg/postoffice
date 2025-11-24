use async_trait::async_trait;
use mlua::prelude::*;
use serde::Deserialize;

use crate::message::InternalMessage;

use super::Block;

#[derive(Debug, Deserialize)]
pub enum LuaFilterConfig {
    Inline(String),
    File(String),
}

pub struct LuaFilterBlock {
    pub script: String,
}

impl LuaFilterBlock {
    pub fn new(config: LuaFilterConfig) -> anyhow::Result<LuaFilterBlock> {
        match config {
            LuaFilterConfig::Inline(script) => Ok(LuaFilterBlock { script }),
            LuaFilterConfig::File(path) => {
                let script = std::fs::read_to_string(path)?;
                Ok(LuaFilterBlock { script })
            }
        }
    }
}

#[async_trait]
impl Block for LuaFilterBlock {
    async fn exec(self: &Self, message: InternalMessage) -> anyhow::Result<Vec<InternalMessage>> {
        let matches = match message.data {
            crate::message::InternalMessageData::Json(ref value) => {
                let lua = Lua::new();

                lua.globals()
                    .set(
                        "topic",
                        lua.to_value(&message.topic)
                            .map_err(|e| anyhow::Error::msg(e.to_string()))?,
                    )
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

                lua.globals()
                    .set(
                        "data",
                        lua.to_value(value)
                            .map_err(|e| anyhow::Error::msg(e.to_string()))?,
                    )
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

                let finish = lua
                    .create_function(move |lua, matches: bool| {
                        return lua.globals().set("_INTERNAL_MATCHES_", matches);
                    })
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

                lua.globals()
                    .set("finish", finish)
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

                lua.load(&self.script)
                    .exec()
                    .map_err(|e| anyhow::Error::msg(e.to_string()))?;

                let matches: bool = lua.globals().get("_INTERNAL_MATCHES_").unwrap_or(false);

                matches
            }
            _ => false,
        };

        if matches {
            Ok(vec![message])
        } else {
            Ok(vec![])
        }
    }
}
