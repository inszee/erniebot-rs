
use std::collections::HashMap;

use crate::errors::ErnieError;
use crate::utils::{build_safe_gurad_url, get_safe_guard_tokens};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE,AUTHORIZATION};
use serde::{Serialize,Deserialize};
use crate::chat::Response;
use serde_json::Value;
use url::Url;

static SAFE_GUARD_API_URL: &str = "https://afd.bj.baidubce.com/rcs/llm/input/analyze";

/** ChatEndpoint is a struct that represents the chat endpoint of erniebot API
*/
#[derive(Debug, Clone)]
pub struct SafeGuardEndpoint {
    url: Url,
    access_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SafeGuardResponse {
    pub request_id: String,
    pub ret_code: String,
    pub ret_msg: String,
    pub ret_data: RetData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetData {
    pub action: i32,
    pub is_safe: i32,
    pub hit_type: String,
    pub sub_hit_type: String,
    pub lang_type: String,

    #[serde(default)]
    pub score: Option<f64>,

    #[serde(default)]
    pub default_answer: Option<String>,
}

impl SafeGuardEndpoint {
    /// create a new chat instance using pre-defined model
    pub fn new(ak: &str,sk: &str) -> Result<Self, ErnieError> {
        Ok(SafeGuardEndpoint {
            url: build_safe_gurad_url(SAFE_GUARD_API_URL)?,
            access_token: get_safe_guard_tokens(ak,sk)?,
        })
    }

    fn generate_body(
        query: String,
        history_qa: Vec<HashMap<String, String>>,
        appid: Option<String>,
        template_id: Option<String>,
        user_id: Option<String>,
        stream: bool
    ) -> Result<serde_json::Value, ErnieError> {
        let mut body = serde_json::json!({
            "query": query,
            "historyQA": history_qa,
            "stream": stream
        });

        if let Some(appid) = appid {
            body["appid"] = serde_json::Value::String(appid);
        }

        if let Some(template_id) = template_id {
            body["templateId"] = serde_json::Value::String(template_id);
        }

        if let Some(user_id) = user_id {
            body["userId"] = serde_json::Value::String(user_id);
        }

        Ok(body)
    }

    /// ainvoke method is used to send a request to erniebot chat endpoint. This is an async method that will return a full response from the chat endpoint
    pub async fn ainvoke(
        &self,
        query: String,
    ) -> Result<Response, ErnieError> {
        let body = SafeGuardEndpoint::generate_body(
            query, vec![], None,Some(r#"custom_musume_template"#.to_string()), None,false)?;

        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        let authrization = self.access_token.clone();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8")
        );
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&authrization).unwrap());
        let authrization = self.access_token.clone();
        let final_url = format!(
            "{}?authorization={}",
            self.url,
            urlencoding::encode(authrization.as_str())
        );

        let response: Value = client
            .post(&final_url)
            .headers(headers)
            .query(&[("authorization",authrization.as_str())])
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
        Ok(Response::new(response))
    }

    pub fn invoke(
        &self,
        query: String,
    ) -> Result<Response, ErnieError> {
        let body = SafeGuardEndpoint::generate_body(
            query, vec![], None,Some(r#"custom_musume_template"#.to_string()), None,false)?;

        let authrization = self.access_token.clone();
        let final_url = format!(
            "{}?authorization={}",
            self.url,
            urlencoding::encode(authrization.as_str())
        );

        println!("final_url={},body = {:#?}", final_url,body);
        match ureq::post(&final_url)
            .set("Content-Type", "application/json")
            .set("Authorization", &authrization)
            .send_json(body)
        {
            Ok(resp) => {
                  let text = resp.into_string().unwrap();
                println!("resp: {}", text);

                let response: Value = serde_json::from_str(&text)
                    .map_err(|e| ErnieError::InvokeError(e.to_string()))?;

                if response.get("error_code").is_some() {
                    return Err(ErnieError::RemoteAPIError(response.to_string()));
                }

                Ok(Response::new(response))
            }
            Err(ureq::Error::Status(code, resp)) => {
                let error_response = resp.into_string().unwrap();
                println!("ureq::Error status={},resonse = {}", code,error_response);
                return Err(ErnieError::InvokeError(error_response))
            }
            Err(e) => {
                println!("unknow err: {:?}", e);
                return Err(ErnieError::InvokeError(e.to_string()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::chat::{safeguard::SafeGuardEndpoint};
    #[test]
    fn test_safe_guard_body() {
        let ak = r#""#.to_string();
        let sk =  r#""#.to_string();
        
        let prompt = r#"臭狗屎"#.to_string();
        let safe_guard_cli = SafeGuardEndpoint::new(&ak,&sk).unwrap();
        match SafeGuardEndpoint::invoke(&safe_guard_cli,prompt) {
            Ok(result) => {
                println!("invoke success : {:#?}", result);
                let s = serde_json::to_string(&result).unwrap();
                
                println!("json: {}", s);
            }
            Err(err) => {
                println!("SafeGuardEndpoint err:{}",err);
            }
        }
    }
}
