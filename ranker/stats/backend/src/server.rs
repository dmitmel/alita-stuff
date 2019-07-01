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

      let mut json_bytes: Vec<u8> = vec![];
      json_bytes.push(b'[');
      for record in &db.records {
        json_bytes.push(b'[');
        itoa::write(&mut json_bytes, record.timestamp).unwrap();
        json_bytes.push(b',');
        itoa::write(&mut json_bytes, record.rank).unwrap();
        json_bytes.push(b',');
        itoa::write(&mut json_bytes, record.upvotes).unwrap();
        json_bytes.push(b',');
        itoa::write(&mut json_bytes, record.downvotes).unwrap();
        json_bytes.push(b',');
        itoa::write(&mut json_bytes, record.reranks).unwrap();
        json_bytes.push(b',');
        itoa::write(&mut json_bytes, record.top5_reranks).unwrap();
        json_bytes.push(b']');
        json_bytes.push(b',');
      }
      json_bytes.pop();
      json_bytes.push(b']');

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
