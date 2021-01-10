use crate::prelude::*;
use isahc::{AsyncBody, AsyncReadResponseExt, Response};
use serde::de;

#[async_trait]
pub trait SurfExt {
    async fn deserialize<T>(mut self) -> Result<T, Error>
    where
        T: de::DeserializeOwned;
}

#[async_trait]
impl SurfExt for Response<AsyncBody> {
    async fn deserialize<T>(mut self) -> Result<T, Error>
    where
        T: de::DeserializeOwned,
    {
        if !self.status().is_success() {
            log::error!("Server responded with error");
            return Err(Error::ServerUnavailable);
        }
        let body = self.text().await.map_err(Error::IOError)?;
        serde_json::from_str(&body).map_err(Error::OAuthParsingError)
    }
}
