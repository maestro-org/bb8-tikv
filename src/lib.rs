//! TiKV support for the `bb8` connection pool.
#![warn(missing_docs)]
pub use bb8;
pub use tikv_client::{Config, Error, RawClient, Result as TiKVResult, TransactionClient};

use async_trait::async_trait;
use bb8::{ManageConnection};

/// A `bb8::ManageConnection` for `tikv_client::RawClient`
#[derive(Clone, Debug)]
pub struct TiKVRawConnectionManager {
    /// Raw client of TiKV
    config: Option<Config>,
    /// Addresses of pd endpoints
    pd_endpoints: Vec<String>,
}

impl TiKVRawConnectionManager {
    /// Create new raw connection manager
    ///
    /// # Arguments
    /// * pd_endpoints - where to connect to pd server(s) (address:port)
    /// * config - optional config of TiKV client
    pub fn new<S>(pd_endpoints: Vec<S>, config: Option<Config>) -> TiKVResult<Self>
    where
        S: Into<String>,
    {
        let mut pd_endpoints = pd_endpoints;
        Ok(Self {
            pd_endpoints: pd_endpoints.drain(..).map(|e| e.into()).collect(),
            config,
        })
    }
}

#[async_trait]
impl ManageConnection for TiKVRawConnectionManager {
    type Error = Error;
    type Connection = RawClient;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        if let Some(config) = &self.config {
            Ok(RawClient::new_with_config(self.pd_endpoints.clone(), config.clone(), None).await?)
        } else {
            Ok(RawClient::new(self.pd_endpoints.clone(), None).await?)
        }
    }

    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.get(String::new()).await?;
        Ok(())
    }

    fn has_broken(&self, _client: &mut Self::Connection) -> bool {
        false
    }
}

/// A `bb8::ManageConnection` for `tikv_client::TransactionClient`
#[derive(Clone, Debug)]
pub struct TiKVTransactionalConnectionManager {
    /// Config of TiKV client
    config: Option<Config>,
    /// Addresses of pd endpoints
    pd_endpoints: Vec<String>,
}

impl TiKVTransactionalConnectionManager {
    /// Create new transactional connection manager
    ///
    /// # Arguments
    /// * pd_endpoints - where to connect to pd server(s) (address:port)
    /// * config - optional config of TiKV client
    pub fn new<S>(pd_endpoints: Vec<S>, config: Option<Config>) -> TiKVResult<Self>
    where
        S: Into<String>,
    {
        let mut pd_endpoints = pd_endpoints;
        Ok(Self {
            pd_endpoints: pd_endpoints.drain(..).map(|e| e.into()).collect(),
            config,
        })
    }
}

#[async_trait]
impl ManageConnection for TiKVTransactionalConnectionManager {
    type Error = Error;
    type Connection = TransactionClient;

    async fn connect(&self) -> Result<Self::Connection, Self::Error> {
        if let Some(config) = &self.config {
            Ok(
                TransactionClient::new_with_config(self.pd_endpoints.clone(), config.clone(), None)
                    .await?,
            )
        } else {
            Ok(TransactionClient::new(self.pd_endpoints.clone(), None).await?)
        }
    }
    async fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.current_timestamp().await?;
        Ok(())
    }

    fn has_broken(&self, _client: &mut Self::Connection) -> bool {
        false
    }
}
