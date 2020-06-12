use crate::bcdb::generated::acl_client::AclClient;
use crate::bcdb::generated::*;
use serde::{Deserialize, Serialize};
use warp::http::StatusCode;
use warp::reject::{Rejection};
use warp::Filter;

type Client = AclClient<tonic::transport::Channel>;

#[derive(Serialize, Deserialize, Debug)]
struct ACLGetResponse {
    perm: String,
    users: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ACLSetRequest {
    perm: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ACLCreateRequest {
    perm: String,
    users: Vec<u64>
}

#[derive(Serialize, Deserialize, Debug)]
struct ACLUsersRequest {
    users: Vec<u64>
}

async fn handle_create(
    cl: Client,
    auth: String,
    body: ACLCreateRequest,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclCreateRequest {
        acl: Some(Acl{
            perm: body.perm,
            users: body.users
        })
    };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.create(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::json(&response.key))
}

async fn handle_set(
    cl: Client,
    auth: String,
    key: u32,
    body: ACLSetRequest,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclSetRequest {
        key: key,
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

async fn handle_grant(
    cl: Client,
    auth: String,
    key: u32,
    body: ACLUsersRequest,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclUsersRequest {
        key: key,
        users: body.users,
    };

    info!("request: {:?}", request);

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.grant(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::json(&response.updated))
}

async fn handle_revoke(
    cl: Client,
    auth: String,
    key: u32,
    body: ACLUsersRequest,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = AclUsersRequest {
        key: key,
        users: body.users,
    };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.revoke(request).await {
        Ok(response) => response,
        Err(status) => return Err(super::status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::json(&response.updated))
}

fn with_client(
    cl: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cl.clone())
}

pub fn router(cl: Client) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    let base = warp::any()
        .and(with_client(cl.clone()))
        .and(warp::header::header::<String>("authorization"));

    let create = base
        .clone()
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_create);
    
    let get = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::get())
        .and_then(handle_get);
        
    let set = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::put())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_set);

    let grant = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::path("grant"))
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_grant);

    let revoke = base
        .clone()
        .and(warp::path::param::<u32>()) // key
        .and(warp::path("revoke"))
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_revoke);

    warp::path("acl")
        .and(create.or(get).or(set).or(grant).or(revoke))
}
