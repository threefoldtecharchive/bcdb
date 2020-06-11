use crate::bcdb::generated::bcdb_client::BcdbClient;
use crate::bcdb::generated::*;
use serde::Serialize;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::Filter;

type Client = BcdbClient<tonic::transport::Channel>;

#[derive(Debug)]
enum BcdbRejection {
    Status(tonic::Status),
}

impl Reject for BcdbRejection {}

async fn handle_set(
    cl: Client,
    collection: String,
    auth: String,
    acl: Option<u32>,
    _tags: Option<String>,
    data: bytes::Bytes,
) -> Result<impl warp::Reply, Rejection> {
    let mut cl = cl.clone();

    let request = SetRequest {
        metadata: Some(Metadata {
            collection: collection,
            tags: Vec::new(),
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
        Err(status) => return Err(status_to_rejection(status)),
    };

    let response = response.into_inner();
    Ok(warp::reply::with_status(
        format!("{}", response.id),
        StatusCode::CREATED,
    ))
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
        Err(status) => return Err(status_to_rejection(status)),
    };
    let response = response.into_inner();

    Ok(warp::reply::with_status(response.data, StatusCode::OK))
}

fn with_client(
    cl: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cl.clone())
}

pub fn router(cl: Client) -> impl Filter<Extract = impl warp::Reply, Error = Infallible> + Clone {
    let base = warp::any()
        .and(with_client(cl.clone()))
        .and(warp::path::param::<String>()) // collection
        .and(warp::header::header::<String>("authorization"));

    let set = base
        .clone()
        .and(warp::post())
        .and(warp::header::optional::<u32>("x-acl"))
        .and(warp::header::optional::<String>("x-tags"))
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::bytes())
        .and_then(handle_set);

    let get = base
        .clone()
        .and(warp::get())
        .and(warp::path::param::<u32>()) // key
        .and_then(handle_get);
    //.map(|cl, collection, auth, key| format!("collection (get): {}\n", collection));

    warp::path("db").and(set.or(get)).recover(handle_rejections)
}

fn status_to_code(status: &tonic::Status) -> StatusCode {
    use tonic::Code::*;
    let code = match status.code() {
        Ok => StatusCode::OK,
        InvalidArgument => StatusCode::BAD_REQUEST,
        DeadlineExceeded => StatusCode::REQUEST_TIMEOUT,
        NotFound => StatusCode::NOT_FOUND,
        AlreadyExists => StatusCode::CONFLICT,
        PermissionDenied => StatusCode::UNAUTHORIZED,
        Unauthenticated => StatusCode::UNAUTHORIZED,
        FailedPrecondition => StatusCode::PRECONDITION_FAILED,
        Unimplemented => StatusCode::NOT_IMPLEMENTED,
        Unavailable => StatusCode::SERVICE_UNAVAILABLE,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    code
}

fn status_to_rejection(status: tonic::Status) -> Rejection {
    return warp::reject::custom(BcdbRejection::Status(status));
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejections(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(BcdbRejection::Status(status)) = err.find() {
        code = status_to_code(status);
        message = status.message();
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed";
    } else {
        // We should have expected this... Just log and say its a 500
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Unknown error";
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
