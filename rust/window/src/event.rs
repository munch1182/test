use libcommon::newerr;
use message::Message;
use tao::window::WindowId;

#[derive(Debug, Default)]
pub enum UserEvent {
    #[default]
    Empty,
    IpcHandle(MessageWithId),
}

pub struct MessageWithId {
    pub id: WindowId,
    pub cmd: String,
    pub payload: Vec<u8>,
}

impl std::fmt::Debug for MessageWithId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MessageWithId")
            .field("id", &self.id)
            .field("cmd", &self.cmd)
            .field("payload", &self.payload.len())
            .finish()
    }
}

impl MessageWithId {
    pub fn new(id: WindowId, msg: Message) -> Self {
        Self {
            id,
            cmd: msg.command,
            payload: msg.payload,
        }
    }
}

#[derive(Debug)]
pub enum SysWindowEvent {
    DragStart,
    Close,
    Minimize,
}

impl TryFrom<String> for SysWindowEvent {
    type Error = libcommon::prelude::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "DragStart" => Ok(Self::DragStart),
            "Close" => Ok(Self::Close),
            "Minimize" => Ok(Self::Minimize),
            _ => Err(newerr!("unkown cmd: {value}")),
        }
    }
}
