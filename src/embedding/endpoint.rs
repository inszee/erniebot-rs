use super::model::EmbeddingModel;
use super::response::EmbeddingResponse;
use crate::errors::ErnieError;
use crate::utils::{build_url, get_access_token};
use json_value_merge::Merge;
use serde_json::Value;
use url::Url;

static EMBEDDING_BASE_URL: &str =
    "https://aip.baidubce.com/rpc/2.0/ai_custom/v1/wenxinworkshop/embeddings/";

pub struct EmbeddingEndpoint {
    url: Url,
    access_token: String,
}

impl EmbeddingEndpoint {
    pub fn new(model: EmbeddingModel) -> Result<Self, ErnieError> {
        Ok(EmbeddingEndpoint {
            url: build_url(EMBEDDING_BASE_URL, model.to_string().as_str())?,
            access_token: get_access_token()?,
        })
    }
    /// sync invoke
    pub fn invoke(
        &self,
        input: Vec<String>,
        user_id: Option<String>,
    ) -> Result<EmbeddingResponse, ErnieError> {
        let mut body = serde_json::json!({
            "input": input,
        });
        if let Some(user_id) = user_id {
            body.merge(&serde_json::json!({"user_id": user_id}));
        }
        let client = reqwest::blocking::Client::new();
        let response: Value = client
            .post(self.url.as_str())
            .query(&[("access_token", self.access_token.as_str())])
            .json(&body)
            .send()
            .map_err(|e| ErnieError::InvokeError(e.to_string()))?
            .json()
            .map_err(|e| ErnieError::InvokeError(e.to_string()))?;

        //if error_code key in response, means RemoteAPIError
        if response.get("error_code").is_some() {
            return Err(ErnieError::RemoteAPIError(response.to_string()));
        }

        Ok(EmbeddingResponse::new(response))
    }
    ///async invoke
    pub async fn ainvoke(
        &self,
        input: Vec<String>,
        user_id: Option<String>,
    ) -> Result<EmbeddingResponse, ErnieError> {
        let mut body = serde_json::json!({
            "input": input,
        });
        if let Some(user_id) = user_id {
            body.merge(&serde_json::json!({"user_id": user_id}));
        }
        let client = reqwest::Client::new();
        let response: Value = client
            .post(self.url.as_str())
            .query(&[("access_token", self.access_token.as_str())])
            .json(&body)
            .send()
            .await
            .map_err(|e| ErnieError::InvokeError(e.to_string()))?
            .json()
            .await
            .map_err(|e| ErnieError::InvokeError(e.to_string()))?;

        //if error_code key in response, means RemoteAPIError
        if response.get("error_code").is_some() {
            return Err(ErnieError::RemoteAPIError(response.to_string()));
        }

        Ok(EmbeddingResponse::new(response))
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::EmbeddingModel;
    use super::EmbeddingEndpoint;
    use tokio::runtime::Runtime;
    #[test]
    fn test_embedding() {
        let embedding = EmbeddingEndpoint::new(EmbeddingModel::EmbeddingV1).unwrap();
        let input = vec![
            "你好".to_string(),
            "你叫什么名字".to_string(),
            "你是谁".to_string(),
        ];
        let embedding_response = embedding.invoke(input, None).unwrap();
        let embedding_results = embedding_response.get_embedding_results().unwrap();
        println!("{},{}", embedding_results.len(), embedding_results[0].len());
    }

    #[test]
    fn test_async_embedding() {
        let embedding = EmbeddingEndpoint::new(EmbeddingModel::EmbeddingV1).unwrap();
        let input = vec![
            "你好".to_string(),
            "你叫什么名字".to_string(),
            "你是谁".to_string(),
        ];
        let rt = Runtime::new().unwrap();
        let embedding_response = rt.block_on(embedding.ainvoke(input, None)).unwrap();
        let embedding_results = embedding_response.get_embedding_results().unwrap();
        println!("{},{}", embedding_results.len(), embedding_results[0].len());
    }
}