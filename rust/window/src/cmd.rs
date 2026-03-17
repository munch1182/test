use crate::MessageWithId;
use dashmap::DashMap;
use message::Message;
use std::pin::Pin;
use wry::WebView;

pub type CommandFn =
    Box<dyn Fn(MessageWithId) -> Pin<Box<dyn Future<Output = Message> + Send>> + Send + Sync>;

#[derive(Default)]
pub(crate) struct CommandHander {
    handers: DashMap<String, CommandFn>,
}

impl CommandHander {
    pub fn register(&self, key: impl Into<String>, handler: CommandFn) {
        self.handers.insert(key.into(), handler);
    }

    pub async fn call(&self, msg: MessageWithId) -> Message {
        let cmd = msg.cmd.clone();
        if let Some(fun) = self.handers.get(&msg.cmd) {
            return fun(msg).await;
        }
        Message::new(cmd, "error")
    }
}

pub(crate) struct CommandResp<'a> {
    webview: &'a WebView,
}

impl<'a> CommandResp<'a> {
    pub fn new(webview: &'a WebView) -> Self {
        Self { webview }
    }

    pub fn resp(&self, msg: Message) -> std::result::Result<(), wry::Error> {
        let str: String = msg.into();
        self.webview
            .evaluate_script(format!("window.resp({str});").as_str())
    }
}
