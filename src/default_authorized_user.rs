use crate::authentication_manager::ServiceAccount;
use crate::prelude::*;
use std::sync::RwLock;
use surf::http::Method;
use surf::Request;

#[derive(Debug)]
pub struct DefaultAuthorizedUser {
    token: RwLock<Token>,
}

impl DefaultAuthorizedUser {
    const DEFAULT_TOKEN_GCP_URI: &'static str = "https://accounts.google.com/o/oauth2/token";
    const USER_CREDENTIALS_PATH: &'static str =
        "/.config/gcloud/application_default_credentials.json";

    pub async fn new() -> Result<Self, Error> {
        let token = RwLock::new(Self::get_token().await?);
        Ok(Self { token })
    }

    fn build_token_request<T: serde::Serialize>(json: &T) -> Request {
        surf::RequestBuilder::new(
            Method::Post,
            surf::Url::parse(Self::DEFAULT_TOKEN_GCP_URI).unwrap(),
        )
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(json).unwrap())
        .build()
    }

    async fn get_token() -> Result<Token, Error> {
        log::debug!("loading user credentials file");
        let home = dirs_next::home_dir().ok_or(Error::NoHomeDir)?;
        let path = home.display().to_string() + Self::USER_CREDENTIALS_PATH;
        log::debug!("filepath: {}", path);
        let cred = UserCredentials::from_file(path).await?;
        let req = Self::build_token_request(&RerfeshRequest {
            client_id: cred.client_id,
            client_secret: cred.client_secret,
            grant_type: "refresh_token".to_string(),
            refresh_token: cred.refresh_token,
        });
        log::debug!("request: {:?}", req);
        surf::client()
            .send(req)
            .await
            .map_err(Error::ConnectionError)?
            .body_json()
            .await
            .map_err(Error::OAuthParsingError)
    }
}

#[async_trait]
impl ServiceAccount for DefaultAuthorizedUser {
    async fn project_id(&self) -> Result<String, Error> {
        Err(Error::NoProjectId)
    }

    fn get_token(&self, _scopes: &[&str]) -> Option<Token> {
        Some(self.token.read().unwrap().clone())
    }

    async fn refresh_token(&self, _scopes: &[&str]) -> Result<Token, Error> {
        let token = Self::get_token().await?;
        *self.token.write().unwrap() = token.clone();
        Ok(token)
    }
}

#[derive(Serialize, Debug)]
struct RerfeshRequest {
    client_id: String,
    client_secret: String,
    grant_type: String,
    refresh_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserCredentials {
    /// Client id
    pub client_id: String,
    /// Client secret
    pub client_secret: String,
    /// Refresh Token
    pub refresh_token: String,
    /// Type
    pub r#type: String,
}

impl UserCredentials {
    async fn from_file<T: AsRef<async_std::path::Path>>(path: T) -> Result<UserCredentials, Error> {
        let content = async_std::fs::read_to_string(path)
            .await
            .map_err(Error::UserProfilePath)?;
        Ok(serde_json::from_str(&content).map_err(Error::UserProfileFormat)?)
    }
}
