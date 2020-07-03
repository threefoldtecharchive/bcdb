use super::PeersList;
use crate::database::*;
use crate::identity::Identity;
use crate::rpc::generated::bcdb_client::BcdbClient;
use crate::rpc::generated::*;
use anyhow::{Context as ErrorContext, Result};
use async_trait::async_trait;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Router<L, P>
where
    L: Database,
    P: PeersList,
{
    local: L,
    peers: P,
    id: Identity,
}

impl<L, P> Router<L, P>
where
    L: Database,
    P: PeersList,
{
    pub fn new(id: Identity, local: L, peers: P) -> Self {
        Router { id, local, peers }
    }

    async fn get_peer(&self, id: u32) -> Result<BcdbClient<tonic::transport::channel::Channel>> {
        let peer = self
            .peers
            .get(id)
            .await
            .map_err(|e| Reason::CannotGetPeer(format!("failed to get peer: {}", e)))?;

        let con = peer
            .connect()
            .await
            .map_err(|e| Reason::CannotGetPeer(format!("failed to connect to peer: {}", e)))?;

        Ok(BcdbClient::new(con))
    }

    fn set_headers<T>(&self, request: &mut tonic::Request<T>) {
        request.metadata_mut().append(
            "authorization",
            tonic::metadata::AsciiMetadataValue::from_str(
                crate::auth::header(&self.id, None).as_ref(),
            )
            .unwrap(),
        );
    }

    async fn remote_set(
        &self,
        _id: u32,
        _collection: String,
        _data: Vec<u8>,
        _meta: Meta,
        _acl: Option<u64>,
    ) -> Result<Key> {
        bail!(Reason::NotSupported)
    }

    async fn remote_get(&self, id: u32, key: Key, collection: String) -> Result<Object> {
        let request = GetRequest {
            id: key,
            collection: collection,
        };

        let mut request = tonic::Request::new(request);
        self.set_headers(&mut request);

        let mut cl = self.get_peer(id).await?;

        let response = cl.get(request).await.map_err(|s| Reason::from(s))?;

        let response = response.into_inner();
        let meta = match response.metadata {
            Some(meta) => Meta::from(meta.tags),
            None => Meta::default(),
        };

        Ok(Object {
            key: key,
            data: Some(response.data),
            meta: meta,
        })
    }

    async fn remote_fetch(&self, id: u32, key: Key) -> Result<Object> {
        let request = FetchRequest { id: key };

        let mut request = tonic::Request::new(request);
        self.set_headers(&mut request);

        let mut cl = self.get_peer(id).await?;

        let response = cl.fetch(request).await.map_err(|s| Reason::from(s))?;

        let response = response.into_inner();
        let meta = match response.metadata {
            Some(meta) => Meta::from(meta.tags),
            None => Meta::default(),
        };

        Ok(Object {
            key: key,
            data: Some(response.data),
            meta: meta,
        })
    }

    async fn remote_delete(&mut self, id: u32, key: Key, collection: String) -> Result<()> {
        let request = DeleteRequest {
            id: key,
            collection: collection,
        };

        let mut request = tonic::Request::new(request);
        self.set_headers(&mut request);

        let mut cl = self.get_peer(id).await?;

        cl.delete(request).await.map_err(|s| Reason::from(s))?;

        Ok(())
    }

    async fn remote_update(
        &self,
        id: u32,
        key: Key,
        collection: String,
        data: Option<Vec<u8>>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<()> {
        let request = UpdateRequest {
            id: key,
            metadata: Some(Metadata {
                tags: meta.into(),
                collection: collection,
                acl: acl.map(|acl| AclRef { acl }),
            }),
            data: data.map(|data| update_request::UpdateData { data }),
        };

        let mut request = tonic::Request::new(request);
        self.set_headers(&mut request);

        let mut cl = self.get_peer(id).await?;

        cl.update(request).await.map_err(|s| Reason::from(s))?;

        Ok(())
    }

    async fn remote_list(
        &self,
        _id: u32,
        _meta: Meta,
        _collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>> {
        bail!(Reason::NotSupported);
    }

    async fn remote_find(
        &self,
        _id: u32,
        _meta: Meta,
        _collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>> {
        bail!(Reason::NotSupported);
    }
}

#[async_trait]
impl<L, P> Database for Router<L, P>
where
    L: Database,
    P: PeersList,
{
    async fn set(
        &mut self,
        ctx: Context,
        collection: String,
        data: Vec<u8>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<Key> {
        match ctx.route {
            Route::Local => self.local.set(ctx, collection, data, meta, acl).await,
            Route::Remote(id) => self.remote_set(id, collection, data, meta, acl).await,
        }
    }

    async fn fetch(&mut self, ctx: Context, key: Key) -> Result<Object> {
        match ctx.route {
            Route::Local => self.local.fetch(ctx, key).await,
            Route::Remote(id) => self.remote_fetch(id, key).await,
        }
    }

    async fn get(&mut self, ctx: Context, key: Key, collection: String) -> Result<Object> {
        match ctx.route {
            Route::Local => self.local.get(ctx, key, collection).await,
            Route::Remote(id) => self.remote_get(id, key, collection).await,
        }
    }

    async fn delete(&mut self, ctx: Context, key: Key, collection: String) -> Result<()> {
        match ctx.route {
            Route::Local => self.local.delete(ctx, key, collection).await,
            Route::Remote(id) => self.remote_delete(id, key, collection).await,
        }
    }

    async fn update(
        &mut self,
        ctx: Context,
        key: Key,
        collection: String,
        data: Option<Vec<u8>>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<()> {
        match ctx.route {
            Route::Local => {
                self.local
                    .update(ctx, key, collection, data, meta, acl)
                    .await
            }
            Route::Remote(id) => {
                self.remote_update(id, key, collection, data, meta, acl)
                    .await
            }
        }
    }

    async fn list(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>> {
        match ctx.route {
            Route::Local => self.local.list(ctx, meta, collection).await,
            Route::Remote(id) => self.remote_list(id, meta, collection).await,
        }
    }

    async fn find(
        &mut self,
        ctx: Context,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>> {
        match ctx.route {
            Route::Local => self.local.find(ctx, meta, collection).await,
            Route::Remote(id) => self.remote_find(id, meta, collection).await,
        }
    }
}
