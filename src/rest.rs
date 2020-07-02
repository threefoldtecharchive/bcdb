/*
Resp adaptor for bcdb. The current implementation uses a grpc client to self
and forward all calls from the HTTP interface to the official grpc interface.

This might change in the future to directly access the data layer
*/
use crate::database::Reason;
use crate::identity::Identity;
use crate::rpc::generated::acl_client::AclClient;
use crate::rpc::generated::bcdb_client::BcdbClient;
use anyhow::Error;
use serde::Serialize;
use std::convert::Infallible;
use tokio::net::UnixListener;
use warp::http::StatusCode;
use warp::reject::{Reject, Rejection};
use warp::Filter;

mod acl;
mod bcdb;

#[derive(Debug)]
enum BcdbRejection {
    Error(Error),
    InvalidTagsString,
}

impl Reject for BcdbRejection {}

pub fn rejection(err: Error) -> Rejection {
    warp::reject::custom(BcdbRejection::Error(err))
}

fn error_to_code(err: &Error) -> StatusCode {
    if let Some(e) = err.downcast_ref::<Reason>() {
        return match e {
            Reason::Unauthorized => StatusCode::UNAUTHORIZED,
            Reason::NotFound => StatusCode::NOT_FOUND,
            Reason::NotSupported => StatusCode::NOT_IMPLEMENTED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
    }

    StatusCode::INTERNAL_SERVER_ERROR
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
    let formatted_error = format!("{:?}", err);

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(BcdbRejection::Error(err)) = err.find() {
        code = error_to_code(&err);
        message = format!("{}", err);
    } else if let Some(BcdbRejection::InvalidTagsString) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid tags header";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed";
    } else {
        // We should have expected this... Just log and say its a 500
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = &formatted_error;
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

pub async fn run(id: Identity, unx: String, grpc: u16) -> Result<(), Error> {
    let u = format!("http://127.0.0.1:{}", grpc);

    let channel = loop {
        let channel = tonic::transport::Endpoint::new(u.clone())?.connect().await;
        match channel {
            Ok(channel) => break channel,
            Err(err) => {
                debug!("failed to establish connection to grpc interface: {}", err);
                debug!("retrying");
                tokio::time::delay_for(std::time::Duration::from_millis(300)).await;
                continue;
            }
        };
    };

    // let channel = tonic::transport::Endpoint::new(u)?.connect().await;
    let bcdb_api = bcdb::router(id.clone(), BcdbClient::new(channel.clone()));
    let acl_api = acl::router(id.clone(), AclClient::new(channel));

    let api = bcdb_api.or(acl_api).recover(handle_rejections);

    let _ = std::fs::remove_file(&unx);
    let mut listener = UnixListener::bind(unx)?;
    let incoming = listener.incoming();

    warp::serve(api).run_incoming(incoming).await;

    Ok(())
}
