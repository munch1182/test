use axum::{
    body::{Body, Bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use http_body_util::BodyExt;
use hyper::HeaderMap;

enum Dire {
    Req,
    Resp,
}

impl std::fmt::Display for Dire {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dire::Req => write!(f, "request"),
            Dire::Resp => write!(f, "response"),
        }
    }
}

pub async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let url = &parts.uri.to_string();
    let header = &parts.headers;
    let method = &parts.method.to_string();
    print_url_and_headers(Dire::Req, url, method, 0, header, 0);
    let bytes = buffer_and_print(Dire::Req, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    println!("---> END");

    let start = std::time::Instant::now();
    let res = next.run(req).await;
    let cost = start.elapsed().as_millis();

    let code = &res.status().as_u16();
    let (parts, body) = res.into_parts();
    let header = &parts.headers;
    print_url_and_headers(Dire::Resp, url, method, *code, header, cost);
    let bytes = buffer_and_print(Dire::Resp, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));
    println!("<--- END");
    Ok(res)
}

fn print_url_and_headers(
    direction: Dire,
    url: &str,
    method: &str,
    code: u16,
    headers: &HeaderMap,
    ms: u128,
) {
    match direction {
        Dire::Req => println!("---> {method} {url}"),
        Dire::Resp => println!("<--- {code} {url} ({ms}ms)"),
    }
    for (key, value) in headers {
        println!("{key}: {value:?}");
    }
}

async fn buffer_and_print<B>(direction: Dire, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };
    if let Ok(body) = std::str::from_utf8(&bytes) {
        if !body.is_empty() {
            println!("{body}");
        }
    }
    Ok(bytes)
}
