use super::errors::GetResponseError;
use serde::{Deserialize, Serialize};
use serde_json::value;
use tokio::sync::mpsc::{self, UnboundedReceiver};
use tokio_stream::Stream;

/// Response is using for non-stream response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Response {
    raw_response: value::Value,
}

impl Response {
    pub fn new(raw_response: value::Value) -> Self {
        Response { raw_response }
    }

    pub fn get_raw_response(&self) -> &value::Value {
        &self.raw_response
    }

    pub fn get(&self, key: &str) -> Option<&value::Value> {
        self.raw_response.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut value::Value> {
        self.raw_response.get_mut(key)
    }

    pub fn get_result(&self) -> Result<String, GetResponseError> {
        match self.raw_response.get("result") {
            Some(result) => match result.as_str() {
                Some(result_str) => Ok(result_str.to_string()),
                None => Err(GetResponseError::Details(
                    "result is not a string".to_string(),
                )),
            },
            None => Err(GetResponseError::Details("result is not found".to_string())),
        }
    }
}

/// Responses is using for sync stream response
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Responses {
    responses: Vec<Response>,
}

impl Responses {
    pub fn from_text(text: String) -> Result<Self, GetResponseError> {
        let parts = text.split("\n\n").collect::<Vec<&str>>();
        let mut result = Vec::new();
        for part in parts {
            if part.is_empty() {
                continue;
            }
            let json_str = part.strip_prefix("data: ");
            match json_str {
                Some(json_str) => {
                    let response: value::Value = serde_json::from_str(json_str)
                        .map_err(|e| GetResponseError::Details(e.to_string()))?;
                    let response = Response::new(response);
                    result.push(response);
                }
                None => {
                    return Err(GetResponseError::Details(format!(
                        "json_str is not found in this part of data: {}",
                        part
                    )))
                }
            }
        }
        Ok(Responses { responses: result })
    }
    pub fn get_results(&self) -> Result<Vec<String>, GetResponseError> {
        let mut result = Vec::new();
        for response in &self.responses {
            result.push(response.get_result()?);
        }
        Ok(result)
    }

    pub fn get_whole_result(&self) -> Result<String, GetResponseError> {
        let mut result = String::new();
        for response in &self.responses {
            result.push_str(&response.get_result()?);
        }
        Ok(result)
    }
}

pub struct StreamResponse {
    receiver: UnboundedReceiver<Response>,
}
impl StreamResponse {
    pub fn new() -> (mpsc::UnboundedSender<Response>, Self) {
        let (sender, receiver) = mpsc::unbounded_channel();
        (sender, Self { receiver })
    }
}

impl Stream for StreamResponse {
    type Item = Response;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
