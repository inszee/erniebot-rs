use super::errors::ErnieError;
use base64::prelude::*;
use image::{DynamicImage, ImageResult};
use serde_json::Value;
use std::env::var;
use std::time::{Duration, SystemTime};
use url::{ParseError, Url};
use hmac::{Hmac, KeyInit as _, Mac};
use sha2::Sha256;

pub fn get_access_token() -> Result<String, ErnieError> {
    let url = "https://aip.baidubce.com/oauth/2.0/token";
    let ak = var("QIANFAN_AK")
        .map_err(|_| ErnieError::GetAccessTokenError("QIANFAN_AK is not set".to_string()))?;
    let sk = var("QIANFAN_SK")
        .map_err(|_| ErnieError::GetAccessTokenError("QIANFAN_SK is not set".to_string()))?;

    let access_token = var("QIANFAN_TOKEN");
    let access_token_time = var("QIANFAN_TOKEN_SET_TIME");
    let current_time = SystemTime::now();

    let accesstoken = if access_token.is_ok() && access_token_time.is_ok() {
        let access_token_time = access_token_time.unwrap();
        let parsed_timestamp = access_token_time.parse::<u64>().expect("Invalid timestamp");
        let parsed_duration = std::time::Duration::from_secs(parsed_timestamp);
        let parsed_time = std::time::UNIX_EPOCH + parsed_duration;
        let duration = current_time.duration_since(parsed_time).unwrap();
        let seconds = duration.as_secs();
        if seconds > 24 * 60 * 60 {
            // 超过24小时
            log::debug!("超过24小时");
            None
        } else {
            // 未超过24小时
            log::debug!("未超过24小时");
            Some(access_token.unwrap())
        }
    } else {
        None
    };

    if accesstoken.is_some() {
        Ok(accesstoken.unwrap())
    } else {
        let res: Value = ureq::post(url)
            .query("grant_type", "client_credentials")
            .query("client_id", ak.as_str())
            .query("client_secret", sk.as_str())
            .call()
            .map_err(|e| ErnieError::GetAccessTokenError(e.to_string()))?
            .into_json()
            .map_err(|e| ErnieError::GetAccessTokenError(e.to_string()))?;
        if let Some(error) = res.get("error") {
            let error_description = res.get("error_description").unwrap();
            Err(ErnieError::GetAccessTokenError(format!(
                "{}: {}",
                error, error_description
            )))
        } else {
            let access_token = res.get("access_token").unwrap().as_str().unwrap();
            std::env::set_var("QIANFAN_TOKEN", access_token);
            let timestamp = current_time
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards");
            // 将时间戳转换为字符串
            let timestamp_string = timestamp.as_secs().to_string();
            std::env::set_var("QIANFAN_TOKEN_SET_TIME", timestamp_string);
            Ok(access_token.to_string())
        }
    }
}

/// Build the url for the chat model
pub fn build_url(url: &str, model: &str) -> Result<Url, ParseError> {
    let base = Url::parse(url)?;
    let joined = base.join(model)?;
    Ok(joined)
}

pub fn build_safe_gurad_url(url: &str) -> Result<Url, ParseError> {
    let base = Url::parse(url)?;
    Ok(base)
}

pub fn base64_to_image(image_string: String) -> ImageResult<DynamicImage> {
    let bytes = BASE64_STANDARD.decode(image_string).unwrap();
    let img = image::load_from_memory(&bytes).unwrap();
    Ok(img)
}

type HmacSha256 = Hmac<Sha256>;

fn hmac_sha256_hex(key: &str, data: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(key.as_bytes()).expect("invalid key");

    mac.update(data.as_bytes());

    let result = mac.finalize();
    let bytes = result.into_bytes();

    hex::encode(bytes)
}

fn hmac_sha256_hex_bytes(key: &[u8], data: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(key).expect("invalid key");

    mac.update(data.as_bytes());

    let result = mac.finalize();
    let bytes = result.into_bytes();

    hex::encode(bytes)
}

pub fn get_safe_guard_tokens(
    ak: &str,
    sk: &str,
) -> Result<String,ErnieError> {
    let path = r#"/rcs/llm/input/analyze"#.to_string();
    let host = r#"afd.bj.baidubce.com"#.to_string();
    let time_zone = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let expiration_in_seconds = 3600;

    let auth_string_prefix = format!(
        "bce-auth-v1/{}/{}/{}",
        ak,
        time_zone,
        expiration_in_seconds
    );

    let canonical_request = format!(
        "POST\n{}\n\nhost:{}",
        path,
        host
    );

    let mut mac =
        HmacSha256::new_from_slice(sk.as_bytes()).unwrap();

    mac.update(auth_string_prefix.as_bytes());

    let signing_key_hex = hex::encode(mac.finalize().into_bytes());

    let mut mac2 =
        HmacSha256::new_from_slice(signing_key_hex.as_bytes()).unwrap();

    mac2.update(canonical_request.as_bytes());

    let signature = hex::encode(mac2.finalize().into_bytes());
    
    let authorization_header = format!(
        "bce-auth-v1/{}/{}/{}/host/{}",
        ak,
        time_zone,
        expiration_in_seconds,
        signature
    );

    println!("authStringPrefix: {}", auth_string_prefix);
    println!("canonicalRequest: {}", canonical_request);
    println!("signingKey: {}", signing_key_hex);
    println!("signature: {}", signature);
    println!("authorizationHeader: {}", authorization_header);

    Ok(authorization_header)
}

#[cfg(test)]
mod tests {
    use super::{get_safe_guard_tokens,get_access_token};
    /// before run the test, you should set the environment variables QIANFAN_AK and QIANFAN_SK
    #[test]
    fn test_get_access_token() {
        let access_token = get_access_token();
        println!("access_token: {:?}", access_token);
    }

     #[test]
    fn test_get_safe_guard_tokens() {
        let ak = r#""#;
        let sk = r#""#;
        let auth = get_safe_guard_tokens(ak,sk).unwrap();
        println!("access_token: {:?}", auth);
    }
}
