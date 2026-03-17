use crate::{MessageWithId, SysWindowEvent, UserEvent, script::setup_script};
use dashmap::DashMap;
use libcommon::prelude::*;
use message::Message;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window as TaoWindow, WindowBuilder, WindowId},
};
use wry::{WebView as WryWebview, WebViewBuilder};

pub struct WindowManager {
    event: EventLoop<UserEvent>,
    windows: DashMap<WindowId, Window>,
    // curr: Cell<Option<WindowId>>,
}

impl Default for WindowManager {
    fn default() -> Self {
        Self {
            event: EventLoopBuilder::with_user_event().build(),
            windows: Default::default(),
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

    pub fn run(self) -> ! {
        let event = self.event;
        event.run(move |event, _, flow| {
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
                Event::UserEvent(UserEvent::IpcHandle(msg)) => {
                    let id = msg.id;
                    let cmd = msg.cmd;
                    if let Ok(sys) = SysWindowEvent::try_from(cmd) {
                        match sys {
                            SysWindowEvent::DragStart => {
                                if let Some(win) = self.windows.get(&id) {
                                    let drag = win.window.drag_window();
                                    debug!("start drag window({id:?}): {}", drag.is_ok());
                                }
                            }
                            SysWindowEvent::Close => {
                                if let Some(remove) = self.windows.remove(&id) {
                                    drop(remove);
                                    debug!("close window({id:?})");
                                }
                                if self.windows.is_empty() {
                                    info!("all windows closed, exit");
                                    *flow = ControlFlow::Exit;
                                }
                            }
                            SysWindowEvent::Minimize => {
                                if let Some(win) = self.windows.get(&id) {
                                    win.window.set_minimized(true);
                                    debug!("minimize window({id:?})");
                                }
                            }
                        }
                    }
                }
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
                match Message::try_from(str) {
                    Ok(msg) => {
                        if proxy
                            .send_event(UserEvent::IpcHandle(MessageWithId::new(id, msg)))
                            .is_err()
                        {
                            warn!("ipc message send error");
                        }
                    }
                    std::result::Result::Err(_) => warn!("ipc message parse Message error"),
                }
            }))
        .build(&window)?;
        Ok(Self { window, webview })
    }

    fn id(&self) -> WindowId {
        self.window.id()
    }
}
