use crate::auth::header;
use crate::bcdb::generated::bcdb_client::BcdbClient;
use crate::bcdb::generated::*;
use crate::identity::Identity;
use failure::Error;
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

type Client = BcdbClient<tonic::transport::Channel>;

fn tags_from_str(s: &str) -> Result<Vec<Tag>, Rejection> {
    let map: HashMap<String, String> = match serde_json::from_str(s) {
        Ok(map) => map,
        Err(_) => {
            return Err(warp::reject::custom(
                super::BcdbRejection::InvalidTagsString,
            ))
        }
    };
    let mut tags = Vec::new();

    for (k, v) in map {
        tags.push(Tag { key: k, value: v });
    }

    Ok(tags)
}

fn tags_to_str(tags: Vec<Tag>) -> Result<String, Error> {
    let mut map = HashMap::new();
    for tag in tags {
        map.insert(tag.key, tag.value);
    }

    Ok(serde_json::to_string(&map)?)
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

async fn handle_set(
    cl: Client,
    id: Identity,
    collection: String,
    route: Option<String>,
    acl: Option<u32>,
    tags: Option<String>,
    data: bytes::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let tags = match tags {
        Some(t) => tags_from_str(t.as_ref())?,
        None => Vec::new(),
    };

    let request = SetRequest {
        metadata: Some(Metadata {
            collection: collection,
            tags: tags,
            acl: acl.map(|val| AclRef { acl: val as u64 }),
        }),
        data: Vec::from(data.as_ref()),
    };

    let mut request = tonic::Request::new(request);
    set_headers(&mut request, id, route);

    let response = match cl.set(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::with_status(
        warp::reply::json(&response.id),
        StatusCode::CREATED,
    ))
}

async fn handle_get(
    cl: Client,
    id: Identity,
    collection: String,
    route: Option<String>,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = GetRequest {
        id: key,
        collection: collection,
    };

    let mut request = tonic::Request::new(request);
    set_headers(&mut request, id, route);

    let response = match cl.get(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };
    let response = response.into_inner();
    let mut builder = ResponseBuilder::new().status(StatusCode::OK);

    if let Some(meta) = response.metadata {
        if let Some(acl) = meta.acl {
            builder = builder.header(HEADER_ACL, acl.acl)
        }

        builder = builder.header(HEADER_TAGS, tags_to_str(meta.tags).unwrap());
    }

    Ok(builder.body(response.data))
}

async fn handle_delete(
    cl: Client,
    id: Identity,
    collection: String,
    route: Option<String>,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = DeleteRequest {
        id: key,
        collection: collection,
    };

    let mut request = tonic::Request::new(request);
    set_headers(&mut request, id, route);

    match cl.delete(request).await {
        Ok(_) => Ok(warp::reply()),
        Err(status) => Err(super::status_to_rejection(status)),
    }
}

async fn handle_update(
    cl: Client,
    id: Identity,
    collection: String,
    route: Option<String>,
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
    collection: String,
    route: Option<String>,
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

fn with_client(
    cl: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cl.clone())
}

fn with_identity(
    id: Identity,
) -> impl Filter<Extract = (Identity,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || id.clone())
}

pub fn router(
    id: Identity,
    cl: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let base = warp::any()
        .and(with_client(cl.clone()))
        .and(with_identity(id.clone()))
        .and(warp::path::param::<String>()) // collection
        .and(warp::header::optional::<String>(HEADER_ROUTE));

    let set = base
        .clone()
        .and(warp::post())
        .and(warp::header::optional::<u32>(HEADER_ACL))
        .and(warp::header::optional::<String>(HEADER_TAGS))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_set);

    let get = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::get())
        .and_then(handle_get);

    let delete = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::delete())
        .and_then(handle_delete);

    let update = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::put())
        .and(warp::header::optional::<u32>(HEADER_ACL))
        .and(warp::header::optional::<String>(HEADER_TAGS))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_update);

    let find = base
        .clone()
        .and(warp::get())
        .and(warp::query::raw()) // query
        .and_then(handle_find);

    warp::path("db").and(set.or(get).or(delete).or(update).or(find))
}
