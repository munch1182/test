use axum::{Json, body::Body, http::Response, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> From<Resp<T>> for Response<Body> {
    fn from(val: Resp<T>) -> Self {
        Json(val).into_response()
    }
}

impl Resp<String> {
    pub fn error<S: ToString>(code: u16, msg: S) -> Self {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }

    pub fn syserr<S: ToString>(msg: S) -> Self {
        Self::error(1, msg)
    }
}

impl<T> Resp<T> {
    pub fn success(data: T) -> Self {
        Self {
            code: 0,
            msg: String::from("success"),
            data: Some(data),
        }
    }

    pub fn is_ok(&self) -> bool {
        self.code == 0
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
