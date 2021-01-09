use crate::prelude::*;
use serde::de;
use surf::Response;

#[async_trait]
pub trait SurfExt {
    async fn deserialize<T>(mut self) -> Result<T, Error>
    where
        T: de::DeserializeOwned;
}

#[async_trait]
impl SurfExt for Response {
    async fn deserialize<T>(mut self) -> Result<T, Error>
    where
        T: de::DeserializeOwned,
    {
        if !self.status().is_success() {
            log::error!("Server responded with error");
            return Err(Error::ServerUnavailable);
        }
        self.body_json::<T>().await.map_err(Error::ParsingError)
    }
}
