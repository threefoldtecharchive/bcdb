use super::BcdbRejection;
use crate::acl::{ACLStorage, Permissions, ACL as ACLObject};
use crate::storage::Storage;
use anyhow::Error;
use hyper::Body;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::iter::FromIterator;
use std::str::FromStr;
use warp::http::StatusCode;
use warp::reject::Rejection;
use warp::Filter;

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
    users: Vec<u64>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ACLUsersRequest {
    users: Vec<u64>,
}

async fn handle_create<S>(
    mut storage: ACLStorage<S>,
    body: ACLCreateRequest,
) -> Result<impl warp::Reply, Rejection>
where
    S: Storage,
{
    let acl = ACLObject {
        perm: Permissions::from_str(body.perm.as_ref())
            .map_err(|_| warp::reject::custom(BcdbRejection::InvalidACLPermission))?,
        users: body.users,
    };

    let key = storage.create(&acl).map_err(|e| super::rejection(e))?;

    Ok(warp::reply::with_status(
        warp::reply::json(&key),
        StatusCode::CREATED,
    ))
}

async fn handle_set<S>(
    mut storage: ACLStorage<S>,
    key: u32,
    body: ACLSetRequest,
) -> Result<impl warp::Reply, Rejection>
where
    S: Storage,
{
    let acl = storage.get(key).map_err(|e| super::rejection(e))?;
    let mut acl = match acl {
        None => return Err(warp::reject::not_found()),
        Some(acl) => acl,
    };

    acl.perm = body
        .perm
        .parse()
        .map_err(|_| warp::reject::custom(BcdbRejection::InvalidACLPermission))?;

    storage.update(key, &acl).map_err(|e| super::rejection(e))?;

    Ok(warp::reply::reply())
}

async fn handle_get<S>(mut storage: ACLStorage<S>, key: u32) -> Result<impl warp::Reply, Rejection>
where
    S: Storage,
{
    let acl = storage.get(key).map_err(|e| super::rejection(e))?;

    let acl = match acl {
        None => return Err(warp::reject::not_found()),
        Some(acl) => acl,
    };

    let response = ACLGetResponse {
        perm: format!("{}", acl.perm),
        users: acl.users,
    };

    Ok(warp::reply::json(&response))
}

async fn handle_grant<S>(
    mut storage: ACLStorage<S>,
    key: u32,
    body: ACLUsersRequest,
) -> Result<impl warp::Reply, Rejection>
where
    S: Storage,
{
    let acl = storage.get(key).map_err(|e| super::rejection(e))?;
    let mut acl = match acl {
        None => return Err(warp::reject::not_found()),
        Some(acl) => acl,
    };

    let mut set: HashSet<u64> = HashSet::from_iter(acl.users);
    let len = set.len();
    body.users.iter().for_each(|u| {
        set.insert(*u);
    });

    let updated = set.len() - len;
    if updated > 0 {
        acl.users = Vec::from_iter(set.into_iter());

        storage.update(key, &acl).map_err(|e| super::rejection(e))?;
    }

    Ok(warp::reply::json(&updated))
}

async fn handle_revoke<S>(
    mut storage: ACLStorage<S>,
    key: u32,
    body: ACLUsersRequest,
) -> Result<impl warp::Reply, Rejection>
where
    S: Storage,
{
    let acl = storage.get(key).map_err(|e| super::rejection(e))?;
    let mut acl = match acl {
        None => return Err(warp::reject::not_found()),
        Some(acl) => acl,
    };

    let mut set: HashSet<u64> = HashSet::from_iter(acl.users);
    let len = set.len();
    body.users.iter().for_each(|u| {
        set.remove(u);
    });

    let updated = len - set.len();
    if updated > 0 {
        acl.users = Vec::from_iter(set.into_iter());

        storage.update(key, &acl).map_err(|e| super::rejection(e))?;
    }

    Ok(warp::reply::json(&updated))
}

#[derive(Serialize)]
struct ACL {
    perm: String,
    users: Vec<u64>,
}

#[derive(Serialize)]
struct ListResult {
    key: u32,
    acl: ACL,
}

async fn handle_list<S>(mut storage: ACLStorage<S>) -> Result<impl warp::Reply, Rejection>
where
    S: Storage + Send + Clone,
{
    //let mut storage = storage.clone();
    let response = storage.list().map_err(|e| super::rejection(e))?;
    let response: Vec<Result<String, Error>> = response
        .map(|item| -> Result<String, Error> {
            let (key, acl) = item?;
            let data = ListResult {
                key: key,
                acl: ACL {
                    perm: format!("{}", acl.perm),
                    users: acl.users,
                },
            };
            Ok(serde_json::to_string(&data)?)
        })
        .collect();

    //TODO: doing a collect here will load all the list from the db.
    //we need to figure out another way to not have to do this and
    //convert this to a stream

    let stream = tokio::stream::iter(response);
    let body = Body::wrap_stream(stream);

    Ok(warp::reply::Response::new(body))
}

fn with_storage<S>(
    storage: ACLStorage<S>,
) -> impl Filter<Extract = (ACLStorage<S>,), Error = std::convert::Infallible> + Clone
where
    S: Storage + Clone + Send + Sync,
{
    warp::any().map(move || storage.clone())
}

pub fn router<S>(
    storage: ACLStorage<S>,
) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone
where
    S: Storage + Clone + Send + Sync,
{
    let base = warp::any().and(with_storage(storage.clone()));

    let create = base
        .clone()
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_create);

    let list = base.clone().and(warp::get()).and_then(handle_list);

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
        .and(warp::path!(u32 / "grant"))
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_grant);

    let revoke = base
        .clone()
        .and(warp::path!(u32 / "revoke"))
        .and(warp::post())
        .and(warp::body::content_length_limit(4 * 1024 * 1024)) // setting a limit of 4MB
        .and(warp::body::json())
        .and_then(handle_revoke);

    warp::path("acl").and(set.or(grant).or(revoke).or(get).or(create).or(list))
}
