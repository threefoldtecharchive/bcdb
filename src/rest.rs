/*
Resp adaptor for bcdb. The current implementation uses a grpc client to self
and forward all calls from the HTTP interface to the official grpc interface.

This might change in the future to directly access the data layer
*/
use crate::acl::ACLStorage;
use crate::database::{Database, Reason};
use crate::storage::Storage;
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
    InvalidTag,
    InvalidACLPermission,
}

impl Reject for BcdbRejection {}

pub fn rejection(err: Error) -> Rejection {
    warp::reject::custom(BcdbRejection::Error(err))
}

fn error_to_code(err: &Error) -> (StatusCode, String) {
    if let Some(e) = err.downcast_ref::<Reason>() {
        return match e {
            Reason::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".into()),
            Reason::NotFound => (StatusCode::NOT_FOUND, "Not Found".into()),
            Reason::NotSupported => (StatusCode::NOT_IMPLEMENTED, "Not Implemented".into()),
            Reason::InvalidTag => (
                StatusCode::BAD_REQUEST,
                "Use of invalid tag string (':' prefix is for internal use)".into(),
            ),
            Reason::CannotGetPeer(m) => (StatusCode::BAD_REQUEST, m.into()),
            Reason::Unknown(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.into()),
        };
    }

    (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", err))
}

/// An API error serializable to JSON.
#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

async fn handle_rejections(err: Rejection) -> Result<impl warp::Reply, Infallible> {
    let code;
    let message: String;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found".into();
    } else if let Some(BcdbRejection::Error(err)) = err.find() {
        let (c, m) = error_to_code(&err);
        code = c;
        message = m;
    //message = &formatted_error;
    } else if let Some(BcdbRejection::InvalidTagsString) = err.find() {
        code = StatusCode::BAD_REQUEST;
        message = "Invalid tags header".into();
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "Method not allowed".into();
    } else {
        // We should have expected this... Just log and say its a 500
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = format!("{:?}", err);
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}

pub async fn run<D, S>(db: D, acl: ACLStorage<S>, unx: String) -> Result<(), Error>
where
    D: Database + Clone,
    S: Storage + Clone + Send + Sync + 'static,
{
    let bcdb_api = bcdb::router(db);
    let acl_api = acl::router(acl);

    let api = bcdb_api.or(acl_api).recover(handle_rejections);
    let _ = std::fs::remove_file(&unx);
    let mut listener = UnixListener::bind(&unx)?;
    let incoming = listener.incoming();
    info!("starting the rest api on {}", unx);
    warp::serve(api).run_incoming(incoming).await;

    Ok(())
}
