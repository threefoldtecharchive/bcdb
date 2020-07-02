use crate::auth::header;
use crate::database::{Authorization, Context, Database, Meta};
use crate::identity::Identity;
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

fn tags_from_str(s: &str) -> Result<Meta, Rejection> {
    let map: HashMap<String, String> = match serde_json::from_str(s) {
        Ok(map) => map,
        Err(_) => {
            return Err(warp::reject::custom(
                super::BcdbRejection::InvalidTagsString,
            ))
        }
    };

    Ok(Meta::from(map))
}

fn tags_to_str(tags: HashMap<String, String>) -> Result<String, Error> {
    Ok(serde_json::to_string(&tags)?)
}

fn set_headers<T>(request: &mut tonic::Request<T>, id: Identity, route: Option<String>) {
    // sign request.
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(header(&id, None).as_ref()).unwrap(),
    );

    if let Some(header) = route {
        request.metadata_mut().append(
            HEADER_ROUTE,
            tonic::metadata::AsciiMetadataValue::from_str(&header).unwrap(),
        );
    };
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
        None => Meta::default(),
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

async fn handle_update(
    cl: Client,
    id: Identity,
    route: Option<String>,
    collection: String,
    key: u32,
    acl: Option<u32>,
    tags: Option<String>,
    data: bytes::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let tags = match tags {
        Some(t) => tags_from_str(t.as_ref())?,
        None => Vec::new(),
    };

    let request = UpdateRequest {
        id: key,
        metadata: Some(Metadata {
            collection: collection,
            tags: tags,
            acl: acl.map(|val| AclRef { acl: val as u64 }),
        }),
        data: if data.len() > 0 {
            Some(update_request::UpdateData {
                data: Vec::from(data.as_ref()),
            })
        } else {
            None
        },
    };

    let mut request = tonic::Request::new(request);
    set_headers(&mut request, id, route);

    match cl.update(request).await {
        Ok(_) => Ok(warp::reply::reply()),
        Err(status) => Err(super::status_to_rejection(status)),
    }
}

#[derive(Serialize)]
struct FindResult {
    id: u32,
    tags: HashMap<String, String>,
    acl: Option<u64>,
}

async fn handle_find(
    cl: Client,
    id: Identity,
    route: Option<String>,
    collection: String,
    query: String,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let query = url::Url::parse(&format!("q:///?{}", query)).unwrap();
    let mut tags = Vec::new();
    for (k, v) in query.query_pairs() {
        if k == "_" {
            // this is a hack because the query::raw()
            // filter does not work if query string is empty
            continue;
        }
        tags.push(Tag {
            key: k.into(),
            value: v.into(),
        });
    }

    let request = QueryRequest {
        collection: collection,
        tags: tags,
    };

    let mut request = tonic::Request::new(request);
    set_headers(&mut request, id, route);

    let response = match cl.find(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };
    let response = response.into_inner();

    use tokio::stream::StreamExt;
    let response = response.map(|entry| -> Result<String, Error> {
        let entry = entry?;
        let meta = entry.metadata.unwrap();
        let data = FindResult {
            id: entry.id,
            tags: {
                let mut map = HashMap::new();
                for tag in meta.tags {
                    map.insert(tag.key, tag.value);
                }
                map
            },
            acl: meta.acl.map(|v| v.acl),
        };

        Ok(serde_json::to_string(&data)?)
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
        .and(with_database(db.clone()))
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
        .and(warp::header::optional::<u32>(HEADER_ACL))
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
        .and(warp::header::optional::<u32>(HEADER_ACL))
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
