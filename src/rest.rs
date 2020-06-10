/*
Resp adaptor for bcdb. The current implementation uses a grpc client to self
and forward all calls from the HTTP interface to the official grpc interface.

This might change in the future to directly access the data layer
*/
use crate::bcdb::generated::bcdb_client::BcdbClient;

mod bcdb;

pub async fn run() {
    let cl = BcdbClient::connect("http://127.0.0.1:50051").await.unwrap();
    let api = bcdb::router(cl);
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}
