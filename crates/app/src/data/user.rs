use crate::get_app_data;
use api_structure::auth::jwt::{Claim, JWTs};
use api_structure::error::ClientError;
use api_structure::now_timestamp;
use base64::engine::general_purpose;
use base64::Engine;
use keyring::Entry;
use log::error;

#[derive(Clone)]
pub struct User {
    pub user_data: Claim,
    access_token: String,
    refresh_token: String,
}

impl User {
    pub fn new(jwts: JWTs) -> Option<Self> {
        let claim = Self::parse_jwt(&jwts.access_token);
        if let Err(err) = &claim {
            error!("{:?}", err);
        }
        let claim = claim.ok()?;
        Some(Self {
            user_data: claim,
            access_token: jwts.access_token,
            refresh_token: jwts.refresh_token,
        })
    }

    pub fn set_token(password: &str) -> Result<(), ClientError> {
        let entry = Entry::new("manread_tokens", "refresh_token").map_err(|e| ClientError {
            message: "failed to create builder".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        entry.set_password(password).map_err(|e| ClientError {
            message: "Failed to set key".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        Ok(())
    }

    pub fn delete_token() -> Result<(), ClientError> {
        //todo: remove later
        return Ok(());
        let entry = Entry::new("manread_tokens", "refresh_token").map_err(|e| ClientError {
            message: "failed to create builder".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        entry.delete_password().map_err(|e| ClientError {
            message: "failed to delete key".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        Ok(())
    }

    pub fn load_token() -> Result<String, ClientError> {
        let entry = Entry::new("manread_tokens", "refresh_token").map_err(|e| ClientError {
            message: "failed to create builder".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        let token = entry.get_password().map_err(|e| ClientError {
            message: "failed to get password".to_string(),
            cause: Some(e.to_string()),
            data: None,
        })?;
        Ok(token)
    }

    fn parse_jwt(s: &str) -> Result<Claim, ClientError> {
        let mut splits = s.split('.');
        splits.next();
        let mut splits = splits.collect::<Vec<_>>();
        splits.pop();
        let text = splits.join(".");
        let str = general_purpose::STANDARD_NO_PAD
            .decode(text)
            .map_err(|e| ClientError {
                message: "Failed to decode jwt".to_string(),
                cause: Some(e.to_string()),
                data: Some(splits.join(".")),
            })?;
        serde_json::from_slice(&str).map_err(|e| ClientError {
            message: "Failed to convert parse Claim".to_string(),
            cause: Some(e.to_string()),
            data: Some(String::from_utf8(str).unwrap()),
        })
    }

    pub async fn get_updated_tokens(refresh_token: &str) -> Option<JWTs> {
        let token: JWTs = get_app_data()
            .client
            .post(get_app_data().url.join("refresh").unwrap())
            .header("Authorization", format!("Bearer {}", refresh_token))
            .send()
            .await
            .ok()?
            .json()
            .await
            .ok()?;
        let _ = Self::set_token(token.refresh_token.as_str());
        Some(token)
    }

    pub async fn get_new_access_token(&mut self) -> Option<String> {
        if self.is_refresh_valid() {
            let new = Self::get_updated_tokens(&self.refresh_token).await;
            if let Some(jwts) = new {
                let user = Self::parse_jwt(&jwts.access_token);
                if user.is_err() {
                    return None;
                }
                self.user_data = user.unwrap();
                self.access_token = jwts.access_token;
                self.refresh_token = jwts.refresh_token;
                return Some(self.refresh_token.clone());
            }
        }
        None
    }

    pub fn get_acces_toke(&self) -> Option<String> {
        if self.is_access_valid() {
            Some(self.access_token.clone())
        } else {
            None
        }
    }

    fn is_access_valid(&self) -> bool {
        self.user_data.exp > now_timestamp().expect("Time went backwards").as_millis()
    }

    fn is_refresh_valid(&self) -> bool {
        if let Ok(v) = Self::parse_jwt(self.refresh_token.as_str()) {
            return v.exp > now_timestamp().expect("Time went backwards").as_millis();
        }
        false
    }
}
