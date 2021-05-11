//! TiKV support for the `bb8` connection pool.
#![warn(missing_docs)]
pub use bb8;
pub use tikv_client::{Config, Error, RawClient, Result as TiKVResult, TransactionClient};

use async_trait::async_trait;
use bb8::{ManageConnection, PooledConnection};

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
            Ok(RawClient::new_with_config(self.pd_endpoints.clone(), config.clone()).await?)
        } else {
            Ok(RawClient::new(self.pd_endpoints.clone()).await?)
        }
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
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
                TransactionClient::new_with_config(self.pd_endpoints.clone(), config.clone())
                    .await?,
            )
        } else {
            Ok(TransactionClient::new(self.pd_endpoints.clone()).await?)
        }
    }

    async fn is_valid(&self, conn: &mut PooledConnection<'_, Self>) -> Result<(), Self::Error> {
        conn.begin_optimistic().await?;
        Ok(())
    }

    fn has_broken(&self, _client: &mut Self::Connection) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{TiKVRawConnectionManager, TiKVTransactionalConnectionManager};
    use bb8::Pool;
    use futures::future::join_all;
    use mock_tikv::{start_mock_pd_server, start_mock_tikv_server, MOCK_PD_PORT};

    #[tokio::test]
    async fn test_raw_manager() {
        let mut pd_server = start_mock_pd_server();
        let mut tikv_server = start_mock_tikv_server();
        let pd_servers = vec![format!("localhost:{}", MOCK_PD_PORT)];

        // build pool
        let manager = TiKVRawConnectionManager::new(pd_servers, None).unwrap();
        let pool = Pool::builder().max_size(10).build(manager).await.unwrap();

        // execute parallel queries
        let clients_fut: Vec<_> = (0..8).into_iter().map(|_| pool.get()).collect();
        let clients: Vec<_> = join_all(clients_fut)
            .await
            .drain(..)
            .map(Result::unwrap)
            .collect();
        let futures: Vec<_> = clients
            .iter()
            .map(|client| client.get(String::new()))
            .collect();

        join_all(futures).await;

        tikv_server.shutdown();
        pd_server.shutdown();
    }

    #[tokio::test]
    async fn test_transactional_manager() {
        let mut pd_server = start_mock_pd_server();
        let mut tikv_server = start_mock_tikv_server();
        let pd_servers = vec![format!("localhost:{}", MOCK_PD_PORT)];

        // build pool
        let manager = TiKVTransactionalConnectionManager::new(pd_servers, None).unwrap();
        let pool = Pool::builder().max_size(10).build(manager).await.unwrap();

        // execute parallel queries
        let clients_fut: Vec<_> = (0..8).into_iter().map(|_| pool.get()).collect();
        let clients: Vec<_> = join_all(clients_fut)
            .await
            .drain(..)
            .map(Result::unwrap)
            .collect();
        let futures: Vec<_> = clients
            .iter()
            .map(|client| async move {
                let mut txn = client.begin_optimistic().await?;
                txn.get(String::new()).await?;
                txn.commit().await?;
                let result: Result<(), tikv_client::Error> = Ok(());
                result
            })
            .collect();

        join_all(futures).await;

        tikv_server.shutdown();
        pd_server.shutdown();
    }
}

#[cfg(doctest)]
mod test_readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }
    external_doc_test!(include_str!("../README.md"));
}
