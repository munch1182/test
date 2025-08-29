use axum::{Json, body::Body, http::Response, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Resp<T> {
    pub code: u16,
    pub msg: String,
    pub data: Option<T>,
}

impl<T: Serialize> Into<Response<Body>> for Resp<T> {
    fn into(self) -> Response<Body> {
        Json(self).into_response()
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

    pub fn error<S: ToString>(code: u16, msg: S) -> Self {
        Self {
            code,
            msg: msg.to_string(),
            data: None,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.code == 0
    }

    pub fn is_err(&self) -> bool {
        !self.is_ok()
    }
}
