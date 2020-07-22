use crate::database::{Authorization, Context, Database};
use anyhow::Error;
use http::response::Builder as ResponseBuilder;
use hyper::Body;
use serde::Serialize;
use std::collections::HashMap;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Filter;

const HEADER_ACL: &str = "x-acl";
const HEADER_TAGS: &str = "x-tags";
const HEADER_ROUTE: &str = "x-threebot-id";

fn tags_from_str(s: &str) -> Result<HashMap<String, String>, Rejection> {
    let map: HashMap<String, String> = match serde_json::from_str(s) {
        Ok(map) => map,
        Err(_) => {
            return Err(warp::reject::custom(
                super::BcdbRejection::InvalidTagsString,
            ))
        }
    };

    Ok(map)
}

fn tags_to_str(tags: HashMap<String, String>) -> Result<String, Error> {
    Ok(serde_json::to_string(&tags)?)
}

async fn handle_set<D: Database>(
    mut db: D,
    route: Option<u32>,
    collection: String,
    acl: Option<u64>,
    tags: Option<String>,
    data: bytes::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    let tags = match tags {
        Some(t) => tags_from_str(t.as_ref())?,
        None => HashMap::default(),
    };

    let key = db
        .set(ctx, collection, Vec::from(data.as_ref()), tags, acl)
        .await
        .map_err(|e| super::rejection(e))?;

    Ok(warp::reply::with_status(
        warp::reply::json(&key),
        StatusCode::CREATED,
    ))
}

async fn handle_get<D: Database>(
    mut db: D,
    route: Option<u32>,
    collection: String,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    let object = db
        .get(ctx, key, collection)
        .await
        .map_err(|e| super::rejection(e))?;

    let mut builder = ResponseBuilder::new().status(StatusCode::OK);

    if let Some(acl) = object.meta.acl() {
        builder = builder.header(HEADER_ACL, acl)
    }

    builder = builder.header(HEADER_TAGS, tags_to_str(object.meta.into()).unwrap());

    match object.data {
        Some(data) => Ok(builder.body(data)),
        None => Ok(builder.body(Vec::default())),
    }
}

async fn handle_fetch<D: Database>(
    mut db: D,
    route: Option<u32>,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    let object = db.fetch(ctx, key).await.map_err(|e| super::rejection(e))?;

    let mut builder = ResponseBuilder::new().status(StatusCode::OK);

    if let Some(acl) = object.meta.acl() {
        builder = builder.header(HEADER_ACL, acl)
    }

    builder = builder.header(HEADER_TAGS, tags_to_str(object.meta.into()).unwrap());

    match object.data {
        Some(data) => Ok(builder.body(data)),
        None => Ok(builder.body(Vec::default())),
    }
}

async fn handle_delete<D: Database>(
    mut db: D,
    route: Option<u32>,
    collection: String,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    db.delete(ctx, key, collection)
        .await
        .map_err(|e| super::rejection(e))?;

    Ok(warp::reply())
}

async fn handle_update<D: Database>(
    mut db: D,
    route: Option<u32>,
    collection: String,
    key: u32,
    acl: Option<u64>,
    tags: Option<String>,
    data: bytes::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    let tags = match tags {
        Some(t) => tags_from_str(t.as_ref())?,
        None => HashMap::default(),
    };

    let data = if data.len() > 0 {
        Some(Vec::from(data.as_ref()))
    } else {
        None
    };

    db.update(ctx, key, collection, data, tags, acl)
        .await
        .map_err(|e| super::rejection(e))?;

    Ok(warp::reply())
}

#[derive(Serialize)]
struct FindResult {
    id: u32,
    tags: HashMap<String, String>,
    acl: Option<u64>,
}

async fn handle_find<D: Database>(
    mut db: D,
    route: Option<u32>,
    collection: String,
    query: String,
) -> Result<impl warp::Reply, Rejection> {
    let ctx = Context::default()
        .with_route(route)
        .with_auth(Authorization::Owner);

    let query = url::Url::parse(&format!("q:///?{}", query)).unwrap();
    let mut meta = HashMap::default();
    for (k, v) in query.query_pairs() {
        if k == "_" {
            // this is a hack because the query::raw()
            // filter does not work if query string is empty
            continue;
        }
        meta.insert(k.into(), v.into());
    }

    let results = db
        .find(ctx, meta, Some(collection))
        .await
        .map_err(|e| super::rejection(e))?;

    use tokio::stream::StreamExt;
    let response = results.map(|entry| -> Result<String, Error> {
        let entry = entry?;
        //let meta = entry.metadata.unwrap();
        let data = FindResult {
            id: entry.key,
            acl: entry.meta.acl(),
            tags: entry.meta.into(),
        };

        Ok(serde_json::to_string(&data)? + "\n")
    });

    let body = Body::wrap_stream(response);

    Ok(warp::reply::Response::new(body))
}

fn with_database<D>(d: D) -> impl Filter<Extract = (D,), Error = std::convert::Infallible> + Clone
where
    D: Database + Clone,
{
    warp::any().map(move || d.clone())
}

pub fn router<D>(db: D) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone
where
    D: Database + Clone,
{
    let base = warp::any()
        .and(with_database(db))
        .and(warp::header::optional::<u32>(HEADER_ROUTE));

    let fetch = base
        .clone()
        .and(warp::path::param::<u32>())
        .and(warp::get())
        .and_then(handle_fetch);

    let collection = base.clone().and(warp::path::param::<String>()); // collection

    let set = collection
        .clone()
        .and(warp::post())
        .and(warp::header::optional::<u64>(HEADER_ACL))
        .and(warp::header::optional::<String>(HEADER_TAGS))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_set);

    let get = collection
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::get())
        .and_then(handle_get);

    let delete = collection
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::delete())
        .and_then(handle_delete);

    let update = collection
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::put())
        .and(warp::header::optional::<u64>(HEADER_ACL))
        .and(warp::header::optional::<String>(HEADER_TAGS))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_update);

    let find = collection
        .clone()
        .and(warp::get())
        .and(warp::query::raw()) // query
        .and_then(handle_find);

    warp::path("db").and(fetch.or(set).or(get).or(delete).or(update).or(find))
}
