use failure::{AsFail, Error, Fallible};

use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use hyper::header::{self, HeaderValue};
use hyper::service::{NewService, Service};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::net::SocketAddr;

use crate::database::Database;

pub fn start(
  shared_db: Arc<RwLock<Database>>,
) -> impl Future<Item = (), Error = ()> {
  let addr: SocketAddr = ([0, 0, 0, 0], 3000).into();

  hyper::Server::bind(&addr)
    .serve(Server { shared_db })
    .map_err(|e| eprintln!("server error: {}", e))
}

pub struct Server {
  pub shared_db: Arc<RwLock<Database>>,
}

impl NewService for Server {
  type ReqBody = hyper::body::Body;
  type ResBody = hyper::body::Body;
  type Error = Error;
  type Service = Handler;
  type InitError = Error;
  type Future = future::FutureResult<Self::Service, Self::InitError>;

  fn new_service(&self) -> Self::Future {
    future::ok(Handler {
      shared_db: self.shared_db.clone(),
    })
  }
}

pub struct Handler {
  shared_db: Arc<RwLock<Database>>,
}

impl Service for Handler {
  type ReqBody = hyper::body::Body;
  type ResBody = hyper::body::Body;
  type Error = Error;
  type Future =
    Box<dyn Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

  fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
    let uri = req.uri();
    let path = uri.path();

    if !path.starts_with('/') {
      return Box::new(future::ok(simple_status_response(
        StatusCode::BAD_REQUEST,
      )));
    }
    let path_segments: Vec<&str> = path[1..].split('/').collect();

    macro_rules! route {
      ($($method:ident => $handler:expr),* $(,)?) => {
        match req.method() {
          $(&Method::$method => $handler,)*
          _ => Ok(simple_status_response(StatusCode::METHOD_NOT_ALLOWED)),
        }
      };
    }

    let handler_result: Fallible<_> = match &path_segments[..] {
      ["ranker", "stats.json"] => route! {
        GET => self.get_json_stats(req),
      },
      ["ranker", "stats.csv"] => route! {
        GET => self.get_csv_stats(req),
      },
      _ => Ok(simple_status_response(StatusCode::NOT_FOUND)),
    };

    Box::new(future::ok(handler_result.unwrap_or_else(|error| {
      crate::print_error(error.context("HTTP request handler error").as_fail());
      simple_status_response(StatusCode::INTERNAL_SERVER_ERROR)
    })))
  }
}

fn simple_status_response(status: StatusCode) -> Response<Body> {
  let mut res = Response::new(Body::empty());
  *res.status_mut() = status;
  res
}

impl Handler {
  fn get_json_stats(&mut self, req: Request<Body>) -> Fallible<Response<Body>> {
    let db = self.shared_db.read().unwrap();

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
    Ok(res)
  }

  fn get_csv_stats(&mut self, req: Request<Body>) -> Fallible<Response<Body>> {
    let db = self.shared_db.read().unwrap();

    let mut csv_bytes: Vec<u8> = vec![];
    csv_bytes.extend_from_slice(
      b"timestamp,rank,upvotes,downvotes,reranks,top5_reranks\n",
    );

    for record in &db.records {
      itoa::write(&mut csv_bytes, record.timestamp).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.rank).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.upvotes).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.downvotes).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.reranks).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.top5_reranks).unwrap();
      csv_bytes.push(b'\n');
    }

    let mut res = Response::new(Body::from(csv_bytes));
    res
      .headers_mut()
      .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    Ok(res)
  }
}
