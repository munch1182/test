use crate::{
    IpcReqWithId, IpcRequest, IpcResponse, Message, SysWindowEvent, UserEvent,
    cmd::{CommandHander, resp2web},
    script::setup_script,
};
use dashmap::DashMap;
use libcommon::prelude::*;
use std::{pin::Pin, sync::Arc};
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::{Window as TaoWindow, WindowBuilder, WindowId},
};
use wry::{WebView as WryWebview, WebViewBuilder};

pub struct WindowManager {
    event: EventLoop<UserEvent>,
    windows: DashMap<WindowId, Window>,
    handlers: Arc<CommandHander>,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            event: EventLoopBuilder::with_user_event().build(),
            windows: Default::default(),
            handlers: Default::default(),
        }
    }
}

impl WindowManager {
    pub fn create_window(&self, title: &str, url: &str) -> Result<WindowId> {
        let window = Window::create_default(title, url, &self.event)?;
        let id = window.id();
        self.windows.insert(id, window);
        Ok(id)
    }

    pub(crate) fn send_event(proxy: &EventLoopProxy<UserEvent>, event: UserEvent) {
        if proxy.send_event(event).is_err() {
            warn!("failed to send event, the event loop has been destroyed");
        }
    }

    ///
    /// 注册事件处理函数
    ///
    ///
    /// 通过[`wry::WebViewBuilder::with_ipc_handler`]接受前端通过`window.ipc.postMessage`发送的消息(该命令由wry注入);
    /// 并将该消息[`String`]格式化为固定格式[`crate::IpcRequest`], 并通过其参数`command`分发到对应的处理函数;
    /// 这个格式由[`crate::script`]注入, `command`参数即调用者使用宏注册的方法名;
    /// 调用该方法得到结果, 并通过[`resp2web`]回复给前端(附上前端发送的`id`参数), 这个回复方法也由[`crate::script`]注入;
    ///
    ///
    pub fn reigster<I, F>(&self, handlers: I)
    where
        I: IntoIterator<Item = (String, F)>,
        F: Fn(Message) -> Pin<Box<dyn Future<Output = Result<Message>> + Send + 'static>>
            + Send
            + Sync
            + 'static,
    {
        for (name, ele) in handlers {
            self.handlers.register(name, Box::new(ele));
        }
    }

    pub fn run(self) -> ! {
        let proxy = Arc::new(self.event.create_proxy());
        self.event.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    window_id, event, ..
                } => match event {
                    WindowEvent::CloseRequested => {
                        debug!("rece {window_id:?} WindowEvent::CloseRequested; cloe window");
                    }
                    WindowEvent::Focused(_gained) => {
                        // debug!("rece {window_id:?} WindowEvent::Focused({gained})");
                    }
                    _ => {}
                },
                Event::UserEvent(event) => match event {
                    UserEvent::Empty => {}
                    UserEvent::IpcHandle(msg) => {
                        let id = &msg.id;
                        if let Ok(sys) = SysWindowEvent::try_from(msg.req.command.as_str()) {
                            match sys {
                                SysWindowEvent::DragStart => {
                                    if let Some(win) = self.windows.get(id) {
                                        let drag = win.window.drag_window();
                                        debug!("start drag window({id:?}): {}", drag.is_ok());
                                    }
                                }
                                SysWindowEvent::Close => {
                                    if let Some(remove) = self.windows.remove(id) {
                                        drop(remove);
                                        debug!("close window({id:?})");
                                    }
                                    if self.windows.is_empty() {
                                        info!("all windows closed, exit");
                                        *flow = ControlFlow::Exit;
                                    }
                                }
                                SysWindowEvent::Minimize => {
                                    if let Some(win) = self.windows.get(id) {
                                        win.window.set_minimized(true);
                                        debug!("minimize window({id:?})");
                                    }
                                }
                            }
                        } else {
                            let handlers = self.handlers.clone();
                            let proxy = proxy.clone();
                            let windowid = msg.id;
                            let msgid = msg.req.id;
                            tokio::spawn(async move {
                                let resp = match handlers.call(msg).await {
                                    Ok(resp) => resp,
                                    std::result::Result::Err(e) => IpcResponse::from(
                                        msgid,
                                        serde_json::json!({"error": e.to_string()}),
                                    ),
                                };
                                Self::send_event(&proxy, UserEvent::RespHandle(windowid, resp));
                            });
                        }
                    }
                    UserEvent::RespHandle(id, resp) => {
                        if let Some(win) = self.windows.get(&id)
                            && resp2web(&win.webview, &resp).is_ok()
                        {
                            return;
                        }
                        warn!("failed to send response to webview({id:?})");
                    }
                },
                _ => {}
            }
        })
    }
}

struct Window {
    window: TaoWindow,
    #[allow(unused)]
    webview: WryWebview,
}

impl Window {
    fn create_default(title: &str, url: &str, event: &EventLoop<UserEvent>) -> Result<Self> {
        Self::create(
            |b| b.with_decorations(false).with_title(title),
            |b| b.with_url(url),
            event,
        )
    }

    fn create<WIN, WEB>(win: WIN, web: WEB, event: &EventLoop<UserEvent>) -> Result<Self>
    where
        WIN: FnOnce(WindowBuilder) -> WindowBuilder,
        WEB: FnOnce(WebViewBuilder) -> WebViewBuilder,
    {
        let proxy = event.create_proxy();
        let window = win(WindowBuilder::new()).build(event)?;
        let id = window.id();
        let webview = web(WebViewBuilder::new()
            .with_initialization_script(setup_script())
            .with_ipc_handler(move |req| {
                let str = req.body().to_string();
                match IpcRequest::try_from(str.as_str()) {
                    Ok(msg) => WindowManager::send_event(
                        &proxy,
                        UserEvent::IpcHandle(IpcReqWithId::new(id, msg)),
                    ),
                    std::result::Result::Err(e) => warn!("ipc message parse error: {str}: ${e:?}"),
                }
            }))
        .build(&window)?;
        Ok(Self { window, webview })
    }

    fn id(&self) -> WindowId {
        self.window.id()
    }
}
