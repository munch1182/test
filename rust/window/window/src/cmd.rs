use crate::{IpcReqWithId, IpcResponse, Message, script::bridge_handler_call};
use dashmap::DashMap;
use libcommon::{
    newerr,
    prelude::{ErrMapperExt, Result},
};
use std::pin::Pin;

pub type Error = Box<dyn std::error::Error>;

type CommandFn = Box<
    dyn Fn(Message) -> Pin<Box<dyn Future<Output = std::result::Result<Message, Error>> + Send>>
        + Send
        + Sync,
>;

#[derive(Default)]
pub(crate) struct CommandHander {
    handers: DashMap<String, CommandFn>,
}

impl CommandHander {
    pub fn register(&self, key: impl Into<String>, handler: CommandFn) {
        self.handers.insert(key.into(), handler);
    }

    ///
    /// 分发命令调用
    pub async fn call(&self, msg: IpcReqWithId) -> Result<IpcResponse> {
        let cmd = &msg.req.command;
        match self.handers.get(cmd) {
            Some(fun) => {
                let payload = fun(msg.req.payload).await.newerr()?;
                Ok(IpcResponse::from(msg.req.id, payload))
            }
            None => Err(newerr!("not found cmd: {cmd}")),
        }
    }
}

///
/// 回复消息给前端
///
/// msg 的消息id要与发送的消息id对应
///
/// script要与注入的代码对应 [`crate::script::setup_script`]
pub fn resp2web(webview: &wry::WebView, msg: &IpcResponse) -> Result<()> {
    let json_str = serde_json::to_string(msg).map_err(|e| newerr!(e))?;
    webview.evaluate_script(&bridge_handler_call(&json_str))?;
    Ok(())
}
