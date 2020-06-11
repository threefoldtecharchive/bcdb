/*
Resp adaptor for bcdb. The current implementation uses a grpc client to self
and forward all calls from the HTTP interface to the official grpc interface.

This might change in the future to directly access the data layer
*/
use crate::bcdb::generated::bcdb_client::BcdbClient;
use serde::Serialize;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::Filter;

mod bcdb;

#[derive(Debug)]
enum BcdbRejection {
    Status(tonic::Status),
}

impl Reject for BcdbRejection {}

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

pub async fn run() {
    let cl = BcdbClient::connect("http://127.0.0.1:50051").await.unwrap();
    let api = bcdb::router(cl).recover(handle_rejections);
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}
