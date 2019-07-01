use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use hyper::header::{self, HeaderValue};
use hyper::service::service_fn_ok;
use hyper::{Body, Request, Response, Server};
use std::net::SocketAddr;

use crate::database::Database;

pub fn start(
  shared_db: Arc<RwLock<Database>>,
) -> impl Future<Item = (), Error = ()> {
  let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();

  let new_service = move || {
    let shared_db = shared_db.clone();
    service_fn_ok(move |req: Request<Body>| -> Response<Body> {
      let db = shared_db.read().unwrap();

      let json_bytes: Vec<u8> = serde_json::to_vec(&db.records).unwrap();

      let mut res = Response::new(Body::from(json_bytes));
      res.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
      );
      res
    })
  };

  Server::bind(&addr)
    .serve(new_service)
    .map_err(|e| eprintln!("server error: {}", e))
}
