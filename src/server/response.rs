use crate::{error::Error, types::Result};
use serde::Serialize;
use serde_json::Value;
use typescript_definitions::TypeScriptify;

/// Server Response
#[derive(Default, Debug, Serialize, TypeScriptify)]
pub struct Response {
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<Error>,
}

impl Response {
    pub fn new<T: Serialize>(v: Result<T>) -> Response {
        let mut response = Response::default();
        match v {
            Ok(data) => response.data = serde_json::to_value(data).unwrap().into(),
            Err(error) => response.error = error.into(),
        };
        response
    }
}
