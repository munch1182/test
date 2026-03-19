use libcommon::newerr;
use serde::{Deserialize, Serialize};
use tao::window::WindowId;

pub(crate) type Message = serde_json::Value;

#[derive(Default)]
pub enum UserEvent {
    #[default]
    Empty,
    /// 收到IPC消息, 转发给Loop处理
    IpcHandle(IpcReqWithId),
    /// 收到IPC消息回复, 转发给Loop回复
    RespHandle(WindowId, IpcResponse),
}

unsafe impl Send for UserEvent {}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct IpcRequest {
    pub id: u64, // 请求id, 由前端生成
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Message>,
}

impl TryFrom<&str> for IpcRequest {
    type Error = libcommon::prelude::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let req: IpcRequest = serde_json::from_str(value)?;
        Ok(req)
    }
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct IpcResponse {
    pub id: u64,
    pub payload: Message,
}

impl IpcResponse {
    pub fn from(id: u64, payload: Message) -> Self {
        Self { id, payload }
    }
}

/// 带窗口 ID 的完整请求
#[derive(Debug)]
pub struct IpcReqWithId {
    pub id: WindowId,
    pub req: IpcRequest,
}

unsafe impl Send for IpcReqWithId {}

impl IpcReqWithId {
    pub fn new(id: WindowId, req: IpcRequest) -> Self {
        Self { id, req }
    }
}

#[derive(Debug)]
pub enum SysWindowEvent {
    DragStart,
    Close,
    Minimize,
}

impl TryFrom<&str> for SysWindowEvent {
    type Error = libcommon::prelude::Err;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "DragStart" => Ok(Self::DragStart),
            "Close" => Ok(Self::Close),
            "Minimize" => Ok(Self::Minimize),
            _ => Err(newerr!("unknown cmd: {value}")),
        }
    }
}
