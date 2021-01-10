use isahc::http;

use crate::authentication_manager::ServiceAccount;
use crate::prelude::*;
use isahc::{prelude::*, Request};
use std::str;
use std::sync::RwLock;

#[derive(Debug)]
pub struct DefaultServiceAccount {
    token: RwLock<Token>,
}

impl DefaultServiceAccount {
    const DEFAULT_PROJECT_ID_GCP_URI: &'static str =
        "http://metadata.google.internal/computeMetadata/v1/project/project-id";
    const DEFAULT_TOKEN_GCP_URI: &'static str = "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token";

    pub async fn new() -> Result<Self, Error> {
        let token = RwLock::new(Self::get_token().await?);
        Ok(Self { token })
    }

    fn build_token_request(uri: &str) -> isahc::http::request::Builder {
        isahc::Request::get(uri).header("Metadata-Flavor", "Google")
    }

    async fn get_token() -> Result<Token, Error> {
        log::debug!("Getting token from GCP instance metadata server");
        let req = Self::build_token_request(Self::DEFAULT_TOKEN_GCP_URI);
        let token = req
            .body(())
            .map_err(Error::OAuthConnectionError)?
            .send_async()
            .await
            .map_err(Error::ConnectionError)?
            .deserialize()
            .await?;
        Ok(token)
    }
}

#[async_trait]
impl ServiceAccount for DefaultServiceAccount {
    async fn project_id(&self) -> Result<String, Error> {
        log::debug!("Getting project ID from GCP instance metadata server");
        let req = Self::build_token_request(Self::DEFAULT_PROJECT_ID_GCP_URI);
        let mut rsp = req
            .body(())
            .map_err(Error::OAuthConnectionError)?
            .send_async()
            .await
            .map_err(Error::ConnectionError)?;

        match rsp.text().await {
            Ok(s) => Ok(s),
            Err(_) => Err(Error::ProjectIdNonUtf8),
        }
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
