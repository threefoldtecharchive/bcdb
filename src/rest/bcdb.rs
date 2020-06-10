use crate::bcdb::generated::bcdb_client::BcdbClient;
use crate::bcdb::generated::*;
use failure::Error;
use std::convert::Infallible;
use warp::http::StatusCode;
use warp::reject;
use warp::Filter;

type Client = BcdbClient<tonic::transport::Channel>;

fn status_to_reply(status: tonic::Status) -> warp::reply::WithStatus<String> {
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

    warp::reply::with_status(String::from(status.message()), code)
}

async fn handle_set(
    cl: Client,
    collection: String,
    auth: String,
    acl: Option<u32>,
    _tags: Option<String>,
) -> Result<impl warp::Reply, Infallible> {
    let mut cl = cl.clone();

    let request = SetRequest {
        metadata: Some(Metadata {
            collection: collection,
            tags: Vec::new(),
            acl: acl.map(|val| AclRef { acl: val as u64 }),
        }),
        data: Vec::new(),
    };

    let mut request = tonic::Request::new(request);
    request.metadata_mut().append(
        "authorization",
        tonic::metadata::AsciiMetadataValue::from_str(&auth).unwrap(),
    );

    let response = match cl.set(request).await {
        Ok(response) => response,
        Err(err) => {
            return Ok(status_to_reply(err));
        }
    };

    let response = response.into_inner();
    Ok(warp::reply::with_status(
        format!("{}", response.id),
        StatusCode::CREATED,
    ))
}

fn with_client(
    cl: Client,
) -> impl Filter<Extract = (Client,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || cl.clone())
}

pub fn router(
    cl: Client,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let set = warp::post()
        .and(with_client(cl.clone()))
        .and(warp::path::param::<String>()) // collection
        .and(warp::header::header::<String>("authorization"))
        .and(warp::header::optional::<u32>("x-acl"))
        .and(warp::header::optional::<String>("x-tags"))
        // .and(warp::body::content_length_limit(4096))
        // .and(warp::body::bytes())
        .and_then(handle_set);
    //return set;
    // .map(|collection, acl, _tags, _data| {
    //     format!("collection (post): {} (acl: {:?})\n", collection, acl)
    // });

    let get = warp::get()
        .and(with_client(cl.clone()))
        .and(warp::path::param::<String>()) // collection
        .map(|cl, collection| format!("collection (get): {}\n", collection));

    warp::path("db").and(set.or(get))
}
