use crate::{UserEvent, script::setup_script};
use dashmap::DashMap;
use libcommon::prelude::*;
use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::{Window as TaoWindow, WindowBuilder, WindowId},
};
use wry::{WebView as WryWebview, WebViewBuilder};

pub struct WindowManager {
    event: EventLoop<UserEvent>,
    windows: DashMap<WindowId, Window>,
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
        self.event.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    *flow = ControlFlow::Exit;
                }
                _ => {}
            }
        })
    }
}

struct Window {
    window: TaoWindow,
    _webview: WryWebview,
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
        let window = win(WindowBuilder::new()).build(event)?;
        let _webview = web(WebViewBuilder::new()
            .with_initialization_script(setup_script())
            .with_ipc_handler(|a| debug!("rece ipc: {a:?}")))
        .build(&window)?;
        Ok(Self { window, _webview })
    }

    fn id(&self) -> WindowId {
        self.window.id()
    }
}
