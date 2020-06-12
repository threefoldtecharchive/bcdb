use crate::bcdb::generated::bcdb_client::BcdbClient;
use crate::bcdb::generated::*;
use failure::Error;
use std::collections::HashMap;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Filter;

const HEADER_ACL: &str = "x-acl";
const HEADER_TAGS: &str = "x-tags";

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

async fn handle_set(
    cl: Client,
    collection: String,
    auth: String,
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
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.set(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::json(&response.id))
}

async fn handle_get(
    cl: Client,
    collection: String,
    auth: String,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = GetRequest {
        id: key,
        collection: collection,
    };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.get(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };
    let response = response.into_inner();
    let mut builder = http::response::Builder::new().status(StatusCode::OK);

    if let Some(meta) = response.metadata {
        if let Some(acl) = meta.acl {
            builder = builder.header(HEADER_ACL, acl.acl)
        }

        builder = builder.header(HEADER_TAGS, tags_to_str(meta.tags).unwrap());
    }

    Ok(builder.body(response.data))
}

async fn handle_update(
    cl: Client,
    collection: String,
    auth: String,
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
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    match cl.update(request).await {
        Ok(_) => Ok(warp::reply::reply()),
        Err(status) => Err(super::status_to_rejection(status)),
    }
}

fn with_client(
    cl: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cl.clone())
}

pub fn router(cl: Client) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let base = warp::any()
        .and(with_client(cl.clone()))
        .and(warp::path::param::<String>()) // collection
        .and(warp::header::header::<String>("authorization"));

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
        .and(warp::get())
        .and(warp::path::param::<u32>()) // key
        .and_then(handle_get);

    let update = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::put())
        .and(warp::header::optional::<u32>(HEADER_ACL))
        .and(warp::header::optional::<String>(HEADER_TAGS))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_update);
    //.map(|cl, collection, auth, key| format!("collection (get): {}\n", collection));

    warp::path("db").and(set.or(get).or(update))
}
