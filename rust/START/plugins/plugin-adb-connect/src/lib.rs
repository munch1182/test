mod adb;
use crate::adb::{ConnectResult, PairInfo};
use libcommon::{
    newerr,
    prelude::{Result, info},
};
use plugin::{PluginInterface, PluginMetadata, PluginTypeCommunicator, export_plugin};
use serde::{Deserialize, Serialize};

pub struct PluginAdbConnect;

#[derive(Debug, Deserialize, Serialize)]
pub enum AdbCommand {
    Connect,
    Disconnect,
    Pair(PairInfo),
    Generate,
    List,
}

pub enum AdbResult {
    Success(String),
    Fail(String),
}

#[tonic::async_trait]
impl PluginTypeCommunicator for PluginAdbConnect {
    type In = AdbCommand;
    type Out = AdbResult;

    async fn call(&self, data: Self::In) -> Result<Self::Out> {
        let result = match data {
            AdbCommand::Connect => adb::connect(),
            AdbCommand::Disconnect => adb::disconnect(),
            AdbCommand::Pair(info) => adb::connect_with_pair(info),
            AdbCommand::List => adb::list(),
            AdbCommand::Generate => {
                let pair = adb::PairInfo::new();
                let str = format!("server: {}, pwd:{}", pair.get_server(), pair.get_pwd());
                return Ok(AdbResult::Success(str));
            }
        };
        Ok(result.into())
    }
}

impl From<AdbResult> for Vec<u8> {
    fn from(value: AdbResult) -> Self {
        match value {
            AdbResult::Success(s) => format!("success: {s}"),
            AdbResult::Fail(s) => s,
        }
        .as_bytes()
        .to_vec()
    }
}

impl TryFrom<Vec<u8>> for AdbCommand {
    type Error = libcommon::prelude::Err;

    fn try_from(value: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        let str = String::from_utf8(value)?.to_lowercase();
        info!("adb command: {str}");
        if str == "connect" {
            return Ok(Self::Connect);
        } else if str == "disconnect" {
            return Ok(Self::Disconnect);
        } else if str == "generate" {
            return Ok(Self::Generate);
        } else if str == "list" {
            return Ok(Self::List);
        } else {
            let split = str.split_whitespace().collect::<Vec<_>>();
            if split.len() == 3 {
                let (f, s, t) = (split[0], split[1], split[2]);
                if f == "pair" {
                    return Ok(Self::Pair(PairInfo::from(s, t)));
                }
            }
        }
        Err(newerr!("unknow adb command: {str}"))
    }
}

impl From<ConnectResult> for AdbResult {
    fn from(value: ConnectResult) -> Self {
        match value {
            ConnectResult::ConnectSuccess => AdbResult::Success("success".to_string()),
            ConnectResult::Err(error) => AdbResult::Fail(format!("{error:?}")),
            ConnectResult::List(items) => AdbResult::Success(items),
        }
    }
}

#[tonic::async_trait]
impl PluginInterface for PluginAdbConnect {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            "plugin_adb_connect",
            "0.1.0",
            "munch1182",
            None,
            plugin::PluginType::Hybrid,
        )
    }
}

export_plugin!(PluginAdbConnect);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() -> std::io::Result<()> {
        let c = C {
            command: AdbCommand::Connect,
        };
        let str = serde_json::to_string_pretty(&c)?.to_lowercase();
        println!("str: {str}");
        let _: C = serde_json::from_str(&str).unwrap();
        Ok(())
    }

    #[derive(Serialize, Deserialize)]
    struct C {
        command: AdbCommand,
    }
}
