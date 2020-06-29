use super::PeersList;
use crate::database::*;
use anyhow::Result;
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
}

impl<L, P> Router<L, P>
where
    L: Database,
    P: PeersList,
{
    pub fn new(local: L, peers: P) -> Self {
        Router { local, peers }
    }

    async fn remote_set(
        &self,
        id: u32,
        collection: String,
        data: Vec<u8>,
        meta: Meta,
        acl: Option<u64>,
    ) -> Result<Key> {
        bail!("unimplemented")
    }

    async fn remote_get(&self, id: u32, key: Key) -> Result<Object> {
        bail!("unimplemented")
    }

    async fn remote_delete(&mut self, id: u32, key: Key) -> Result<()> {
        bail!("unimplemented");
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
        bail!("unimplemented");
    }

    async fn remote_list(
        &self,
        id: u32,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>> {
        bail!("unimplemented");
    }

    async fn remote_find(
        &self,
        id: u32,
        meta: Meta,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>> {
        bail!("unimplemented");
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

    async fn get(&mut self, ctx: Context, key: Key) -> Result<Object> {
        match ctx.route {
            Route::Local => self.local.get(ctx, key).await,
            Route::Remote(id) => self.remote_get(id, key).await,
        }
    }

    async fn delete(&mut self, ctx: Context, key: Key) -> Result<()> {
        match ctx.route {
            Route::Local => self.local.delete(ctx, key).await,
            Route::Remote(id) => self.remote_delete(id, key).await,
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
