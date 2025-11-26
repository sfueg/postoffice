use std::vec;

use anyhow::Context;
use bytes::Bytes;
use rosc::{OscArray, OscType};
use serde_json::{Number, Value};

#[derive(Debug, Clone)]
pub struct InternalMessage {
    pub source_connector_idx: usize,
    pub topic: String,
    pub data: InternalMessageData,
}

#[derive(Debug, Clone)]
pub enum InternalMessageData {
    Empty,
    String(String),
    Binary(Bytes),
    Json(Value),
    OSC(Vec<OscType>),
}

impl InternalMessageData {
    pub fn to_empty(self: Self) -> anyhow::Result<Self> {
        Ok(Self::Empty)
    }

    pub fn to_string(self: Self) -> anyhow::Result<Self> {
        match self {
            Self::Empty => Ok(Self::String("".to_string())),
            Self::String(value) => Ok(Self::String(value)),
            Self::Binary(bytes) => Ok(Self::String(Self::binary_to_string(bytes)?)),
            Self::Json(value) => Ok(Self::String(serde_json::to_string(&value)?)),
            Self::OSC(args) => todo!(),
        }
    }

    pub fn to_json(self: Self) -> anyhow::Result<Self> {
        match self {
            Self::Empty => Ok(Self::Json(Value::Null)),
            Self::String(value) => Ok(Self::Json(serde_json::from_str(value.as_str())?)),
            Self::Binary(bytes) => Ok(Self::Json(serde_json::from_str(
                Self::binary_to_string(bytes)?.as_str(),
            )?)),
            Self::Json(value) => Ok(Self::Json(value)),
            Self::OSC(args) => Ok(Self::Json(Self::osc_args_to_json(args)?)),
        }
    }

    pub fn to_binary(self: Self) -> anyhow::Result<Self> {
        match self {
            Self::Empty => Ok(Self::Binary(Bytes::new())),
            Self::String(value) => Ok(Self::Binary(Self::string_to_binary(value)?)),
            Self::Binary(bytes) => Ok(Self::Binary(bytes)),
            Self::Json(value) => Ok(Self::Binary(Self::string_to_binary(
                serde_json::to_string(&value)?,
            )?)),
            Self::OSC(args) => todo!(),
        }
    }

    pub fn to_osc(self: Self) -> anyhow::Result<Self> {
        match self {
            InternalMessageData::Empty => Ok(InternalMessageData::OSC(vec![])),
            InternalMessageData::String(value) => {
                Ok(InternalMessageData::OSC(vec![OscType::String(value)]))
            }
            InternalMessageData::Binary(bytes) => todo!(),
            InternalMessageData::Json(value) => {
                Ok(InternalMessageData::OSC(Self::json_array_to_osc(value)?))
            }
            InternalMessageData::OSC(osc_types) => Ok(InternalMessageData::OSC(osc_types)),
        }
    }

    pub fn get_binary(self: Self) -> anyhow::Result<Bytes> {
        let bin = self.to_binary()?;

        match bin {
            InternalMessageData::Binary(bytes) => Ok(bytes),
            _ => Err(anyhow::Error::msg("Expected binary after conversion")),
        }
    }

    pub fn get_osc(self: Self) -> anyhow::Result<Vec<OscType>> {
        let bin = self.to_osc()?;

        match bin {
            InternalMessageData::OSC(osc_types) => Ok(osc_types),
            _ => Err(anyhow::Error::msg("Expected OSC after conversion")),
        }
    }

    fn string_to_binary(str: String) -> anyhow::Result<Bytes> {
        let bytes = str.as_bytes().to_vec();

        Ok(Bytes::from(bytes))
    }

    fn binary_to_string(bytes: Bytes) -> anyhow::Result<String> {
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    fn json_array_to_osc(value: Value) -> anyhow::Result<Vec<OscType>> {
        match value {
            Value::Array(values) => Ok(values
                .into_iter()
                .map(|value| Self::json_to_osc(value))
                .collect::<anyhow::Result<Vec<_>>>()?),
            _ => Err(anyhow::Error::msg(
                "Can only convert JSON Array to OSC at the top level",
            )),
        }
    }

    fn json_to_osc(value: Value) -> anyhow::Result<OscType> {
        match value {
            Value::Array(values) => Ok(OscType::Array(OscArray {
                content: values
                    .into_iter()
                    .map(|value| Self::json_to_osc(value))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            })),
            Value::Null => Ok(OscType::Nil),
            Value::Bool(value) => Ok(OscType::Bool(value)),
            Value::Number(number) => {
                if number.is_f64() {
                    Ok(OscType::Float(number.as_f64().context("")? as f32))
                } else {
                    Ok(OscType::Int(number.as_i64().context("")? as i32))
                }
            }
            Value::String(value) => Ok(OscType::String(value)),
            Value::Object(_) => Err(anyhow::Error::msg("Can't convert JSON Object to OSC")),
        }
    }

    fn osc_args_to_json(args: Vec<OscType>) -> anyhow::Result<Value> {
        return Ok(Value::Array(
            args.into_iter()
                .map(|value| Self::osc_to_json(value))
                .collect::<anyhow::Result<Vec<_>>>()?,
        ));
    }

    fn osc_to_json(value: OscType) -> anyhow::Result<Value> {
        match value {
            OscType::Int(value) => Ok(Value::Number(Number::from(value))),
            OscType::Float(value) => Ok(Value::Number(
                Number::from_f64(value as f64).context("Failed to get number from f64")?,
            )),
            OscType::String(value) => Ok(Value::String(value)),
            OscType::Bool(value) => Ok(Value::Bool(value)),
            OscType::Char(value) => Ok(Value::String(value.to_string())),
            OscType::Long(value) => Ok(Value::Number(Number::from(value))),
            OscType::Double(value) => Ok(Value::Number(
                Number::from_f64(value).context("Failed to get number from f64")?,
            )),
            OscType::Array(osc_array) => Ok(Self::osc_args_to_json(osc_array.content)?),
            OscType::Nil => Ok(Value::Null),
            OscType::Inf => todo!(),
            OscType::Color(_) => Err(anyhow::Error::msg("Can't convert OSC Color to JSON")),
            OscType::Midi(_) => Err(anyhow::Error::msg("Can't convert OSC Midi to JSON")),
            OscType::Blob(_) => Err(anyhow::Error::msg("Can't convert OSC Blob to JSON")),
            OscType::Time(_) => Err(anyhow::Error::msg("Can't convert OSC Time to JSON")),
        }
    }
}
