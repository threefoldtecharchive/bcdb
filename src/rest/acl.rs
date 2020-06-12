use crate::bcdb::generated::acl_client::AclClient;
use crate::bcdb::generated::*;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::Filter;

type Client = AclClient<tonic::transport::Channel>;

#[derive(Serialize, Deserialize, Debug)]
struct ACLGetResponse {
    perm: String,
    users: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ACLSetRequest {
    key: u32,
    perm: String,
}

async fn handle_set(
    cl: Client,
    collection: String,
    auth: String,
    body: ACLSetRequest,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclSetRequest {
        key: body.key,
        perm: body.perm,
    };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    match cl.set(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    Ok(warp::reply::with_status("created", StatusCode::CREATED))
}

async fn handle_get(
    cl: Client,
    collection: String,
    auth: String,
    key: u32,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclGetRequest { key: key };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    match cl.get(request).await {
        Ok(response) => {
            let response = response.into_inner();

            if let Some(acl) = response.acl {
                let act_get_response = ACLGetResponse {
                    perm: acl.perm,
                    users: acl.users,
                };
                return Ok(warp::reply::json(&act_get_response));
            } else {
                return Err(super::status_to_rejection(tonic::Status::not_found(
                    "acl not found",
                )));
            }
        }
        Err(status) => return Err(super::status_to_rejection(status)),
    };
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
        .and(warp::body::json())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and_then(handle_set);

    let get = base
        .clone()
        .and(warp::get())
        .and(warp::path::param::<u32>()) // key
        .and_then(handle_get);
    //.map(|cl, collection, auth, key| format!("collection (get): {}\n", collection));

    warp::path("acl").and(set.or(get))
}
