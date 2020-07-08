use super::*;
use crate::acl::*;
use crate::storage::Storage;
use anyhow::Context as ErrorContext;
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio::task::spawn_blocking;

//TODO: use generics for both object store type and meta factory type.
#[derive(Clone)]
pub struct BcdbDatabase<S, I>
where
    S: Storage,
    I: Index,
{
    data: S,
    meta: I,
    acl: ACLStorage<S>,
}

impl<S, I> BcdbDatabase<S, I>
where
    S: Storage,
    I: Index + Clone,
{
    pub fn new(data: S, meta: I, acl: ACLStorage<S>) -> Self {
        BcdbDatabase {
            data: data,
            meta: meta,
            acl: acl,
        }
    }

    fn get_permissions(&self, acl: u64, user: u64) -> Result<Permissions> {
        // self.acl.g
        let mut store = self.acl.clone();
        let acl = match store.get(acl as u32)? {
            Some(acl) => acl,
            None => return Ok(Permissions::default()),
        };

        if acl.users.contains(&user) {
            return Ok(acl.perm);
        }

        Ok(Permissions::default())
    }

    fn is_authorized(&self, ctx: &Context, meta: &Meta, perm: Permissions) -> Result<()> {
        match ctx.authorization {
            Authorization::Owner => Ok(()),
            Authorization::User(user) => {
                if let Some(acl) = meta.acl() {
                    let stored = self
                        .get_permissions(acl, user as u64)
                        .context("failed to get assigned permissions")?;

                    if stored.grants(perm) {
                        return Ok(());
                    }
                }

                bail!(Reason::Unauthorized);
            }
            Authorization::Invalid => bail!(Reason::Unauthorized),
        }
    }
}

#[tonic::async_trait]
impl<S, I> Database for BcdbDatabase<S, I>
where
    S: Storage + Send + Sync + 'static,
    I: Index + Clone,
{
    async fn set(
        &mut self,
        ctx: Context,
        collection: String,
        data: Vec<u8>,
        tags: HashMap<String, String>,
        acl: Option<u64>,
    ) -> Result<Key> {
        if !ctx.is_owner() {
            bail!(Reason::Unauthorized)
        }

        let mut meta = Meta::try_from(tags)?;
        if let Some(acl) = acl {
            meta = meta.with_acl(acl);
        }

        meta = meta
            .with_collection(collection)
            .with_size(data.len() as u64)
            .with_created(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );

        let mut db = self.data.clone();
        let id = spawn_blocking(move || db.set(None, &data).expect("failed to set data"))
            .await
            .context("failed to run blocking task")?;

        let mut index = self.meta.clone();
        index.set(id, meta).await.map(|_| id)
    }

    async fn fetch(&mut self, ctx: Context, key: Key) -> Result<Object> {
        let meta = self.meta.get(key).await?;

        self.is_authorized(&ctx, &meta, "r--".parse().unwrap())?;

        let mut db = self.data.clone();
        let data = spawn_blocking(move || db.get(key))
            .await
            .context("failed to run blocking task")?
            .context("failed to get data")?;
        if data.is_none() {
            bail!(Reason::NotFound);
        }

        Ok(Object {
            key: key,
            data: Some(data.unwrap()),
            meta: meta,
        })
    }

    async fn get(&mut self, ctx: Context, key: Key, collection: String) -> Result<Object> {
        let meta = self.meta.get(key).await?;

        if !meta.is_collection(collection) {
            bail!(Reason::NotFound);
        }

        self.is_authorized(&ctx, &meta, "r--".parse().unwrap())?;

        let mut db = self.data.clone();
        let data = spawn_blocking(move || db.get(key))
            .await
            .context("failed to run blocking task")?
            .context("failed to get data")?;
        if data.is_none() {
            bail!(Reason::NotFound);
        }

        Ok(Object {
            key: key,
            data: Some(data.unwrap()),
            meta: meta,
        })
    }

    async fn delete(&mut self, ctx: Context, key: Key, collection: String) -> Result<()> {
        let meta = self.meta.get(key).await?;

        if !meta.is_collection(collection) {
            bail!(Reason::NotFound);
        }

        self.is_authorized(&ctx, &meta, "--d".parse().unwrap())?;

        let mut index = self.meta.clone();
        let meta = Meta::default().with_deleted(true);

        index.set(key, meta).await?;

        Ok(())
    }

    async fn update(
        &mut self,
        ctx: Context,
        key: Key,
        collection: String,
        data: Option<Vec<u8>>,
        tags: HashMap<String, String>,
        acl: Option<u64>,
    ) -> Result<()> {
        let current = self.meta.get(key).await?;

        self.is_authorized(&ctx, &current, "-w-".parse().unwrap())?;

        if !current.is_collection(collection) {
            bail!(Reason::NotFound);
        }

        let mut meta = Meta::try_from(tags)?;
        if let Some(acl) = acl {
            if !ctx.is_owner() {
                bail!(Reason::Unauthorized);
            }

            meta = meta.with_acl(acl);
        }

        meta = meta.with_updated(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        if let Some(data) = data {
            meta = meta.with_size(data.len() as u64);
            let mut db = self.data.clone();
            spawn_blocking(move || db.set(None, &data).expect("failed to set data"))
                .await
                .context("failed to run blocking task")?;
        }

        let mut index = self.meta.clone();
        index.set(key, meta).await?;

        Ok(())
    }

    async fn list(
        &mut self,
        ctx: Context,
        tags: HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Key>>> {
        if !ctx.is_owner() {
            bail!(Reason::Unauthorized);
        }

        let mut meta = Meta::new(tags);

        if let Some(collection) = collection {
            meta.insert(TAG_COLLECTION, collection);
        }

        let mut index = self.meta.clone();
        index.find(meta).await
    }

    async fn find(
        &mut self,
        ctx: Context,
        tags: HashMap<String, String>,
        collection: Option<String>,
    ) -> Result<mpsc::Receiver<Result<Object>>> {
        if !ctx.is_owner() {
            bail!(Reason::Unauthorized);
        }

        let mut meta = Meta::new(tags);

        if let Some(collection) = collection {
            meta.insert(TAG_COLLECTION, collection);
        }

        let mut index = self.meta.clone();

        let (mut tx, rx) = mpsc::channel(10);
        tokio::spawn(async move {
            let mut rx = match index.find(meta).await {
                Ok(rx) => rx,
                Err(err) => {
                    tx.send(Err(anyhow!("{}", err))).await.unwrap();
                    return;
                }
            };

            while let Some(id) = rx.recv().await {
                let id = match id {
                    Ok(id) => id,
                    Err(err) => {
                        tx.send(Err(err)).await.unwrap();
                        return;
                    }
                };

                let meta = match index.get(id).await {
                    Ok(meta) => meta,
                    Err(err) => {
                        tx.send(Err(err)).await.unwrap();
                        return;
                    }
                };

                tx.send(Ok(Object {
                    key: id,
                    meta: meta,
                    data: None,
                }))
                .await
                .unwrap();
            }
        });

        Ok(rx)
    }
}
