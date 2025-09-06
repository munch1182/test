use axum::{Router, routing::get};
use libcommon::{curr_dir, newerr, prelude::*};
use tao::{
    event::{Event, StartCause, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use wry::WebViewBuilder;

#[logsetup]
#[tokio::main]
async fn main() -> Result<()> {
    let host = "127.0.0.1:3000";
    let event_loop = EventLoopBuilder::with_user_event().build();
    let event = event_loop.create_proxy();
    // 服务器和window都会阻塞线程
    tokio::spawn(async { net(host, event).await });
    window(host, event_loop)?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
enum UserEvent {
    ServerReady,
}

async fn net(host: &str, event: EventLoopProxy<UserEvent>) -> Result<()> {
    let lis = TcpListener::bind(host).await?;
    let path = curr_dir!("tauri_impl", "page")?;
    info!("serve at: {path:?}");
    info!("listening on: http://{host}/index");
    let app = Router::new().nest_service(
        "/index",
        ServeDir::new(path).not_found_service(get(async || "404")),
    );
    event.send_event(UserEvent::ServerReady)?;
    axum::serve(lis, app).await?;
    Ok(())
}

fn window(host: &str, event_loop: EventLoop<UserEvent>) -> Result<()> {
    let window = WindowBuilder::new()
        .with_title(host)
        .build(&event_loop)
        .map_err(|e| newerr!("{:?}", e))?;
    let url = format!("http://{host}/index");
    let webview = WebViewBuilder::new()
        .build(&window)
        .map_err(|e| newerr!("{:?}", e))?;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::UserEvent(UserEvent::ServerReady) => {
                info!("server ready");
                let _ = webview.load_url(&url);
            }
            Event::NewEvents(StartCause::Init) => info!("window state: init"),
            _ => {}
        }
    });
}
