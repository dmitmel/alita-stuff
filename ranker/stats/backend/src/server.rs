use failure::{AsFail, Error, Fallible};
use log::info;

use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use hyper::header::{self, HeaderValue};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, Service};
use hyper::{Body, Method, Request, Response, StatusCode};
use std::net::SocketAddr;

use crate::database::Database;
use crate::shutdown::Shutdown;
use crate::tracker;

type HttpRequest = Request<Body>;
type HttpResponse = Response<Body>;

pub fn start(
  config: crate::config::ServerConfig,
  shared_db: Arc<RwLock<Database<tracker::DataPoint>>>,
  shutdown: Shutdown,
) -> impl Future<Item = (), Error = ()> {
  info!("starting on {}", config.address);

  let make_service = make_service_fn(move |socket: &AddrStream| {
    future::ok::<Handler, Error>(Handler {
      remote_addr: socket.remote_addr(),
      shared_db: shared_db.clone(),
    })
  });

  let server = hyper::Server::bind(&config.address)
    .serve(make_service)
    .with_graceful_shutdown(shutdown);
  server.map_err(|e| log_error!(log::Level::Error, e.as_fail())).then(|r| {
    info!("stopping");
    r
  })
}

pub struct Handler {
  remote_addr: SocketAddr,
  shared_db: Arc<RwLock<Database<tracker::DataPoint>>>,
}

impl Service for Handler {
  type ReqBody = hyper::body::Body;
  type ResBody = hyper::body::Body;
  type Error = Error;
  type Future =
    Box<dyn Future<Item = Response<Self::ResBody>, Error = Self::Error> + Send>;

  fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
    use std::time::{Duration, Instant};
    let start_time = Instant::now();

    let method = req.method();
    let uri = req.uri();
    let version = req.version();
    let headers = req.headers();

    let empty_header_value = HeaderValue::from_static("");
    info!(
      r#"{} "{} {} {:?}" {:?} {:?}"#,
      self.remote_addr,
      method,
      uri,
      req.version(),
      headers.get(header::REFERER).unwrap_or(&empty_header_value),
      headers.get(header::USER_AGENT).unwrap_or(&empty_header_value),
    );

    let path = uri.path();
    let res: HttpResponse = if !path.starts_with('/') {
      simple_status_response(StatusCode::BAD_REQUEST)
    } else {
      let path_segments: Vec<&str> = path[1..].split('/').collect();

      macro_rules! route {
        ($($method:ident => $handler:expr),* $(,)?) => {
          match method {
            $(&Method::$method => $handler,)*
            _ => Ok(simple_status_response(StatusCode::METHOD_NOT_ALLOWED)),
          }
        };
      }

      let handler_result: Fallible<_> = match &path_segments[..] {
        ["ranker", "stats.json"] => route! {
          GET => self.get_json_stats(&req),
        },
        ["ranker", "stats.csv"] => route! {
          GET => self.get_csv_stats(&req),
        },
        _ => Ok(simple_status_response(StatusCode::NOT_FOUND)),
      };

      handler_result.unwrap_or_else(|error| {
        log_error!(log::Level::Warn, &error.context("request handler error"));
        simple_status_response(StatusCode::INTERNAL_SERVER_ERROR)
      })
    };

    let elapsed_time: Duration = start_time.elapsed();

    let status = res.status();
    info!(
      r#"{} "{} {} {:?}" "{} {}" {}"#,
      self.remote_addr,
      method,
      uri,
      version,
      status.as_u16(),
      status.canonical_reason().unwrap_or(""),
      elapsed_time.as_micros() as f64 / 1000.0,
    );

    Box::new(future::ok(res))
  }
}

fn simple_status_response(status: StatusCode) -> Response<Body> {
  let mut res = Response::new(Body::empty());
  *res.status_mut() = status;
  res
}

impl Handler {
  fn get_json_stats(&mut self, _req: &HttpRequest) -> Fallible<HttpResponse> {
    let db = self.shared_db.read().unwrap();

    let mut json_bytes: Vec<u8> = vec![];
    json_bytes.push(b'[');

    db.compress_records(|record| {
      json_bytes.push(b'[');
      itoa::write(&mut json_bytes, record.timestamp.as_secs()).unwrap();
      json_bytes.push(b',');
      itoa::write(&mut json_bytes, record.data.rank).unwrap();
      json_bytes.push(b',');
      itoa::write(&mut json_bytes, record.data.upvotes).unwrap();
      json_bytes.push(b',');
      itoa::write(&mut json_bytes, record.data.downvotes).unwrap();
      json_bytes.push(b',');
      itoa::write(&mut json_bytes, record.data.reranks).unwrap();
      json_bytes.push(b',');
      itoa::write(&mut json_bytes, record.data.top5_reranks).unwrap();
      json_bytes.push(b']');
      json_bytes.push(b',');
    });

    json_bytes.pop();
    json_bytes.push(b']');

    let mut res = Response::new(Body::from(json_bytes));
    res.headers_mut().insert(
      header::CONTENT_TYPE,
      HeaderValue::from_static("application/json"),
    );
    Ok(res)
  }

  fn get_csv_stats(&mut self, _req: &HttpRequest) -> Fallible<HttpResponse> {
    let db = self.shared_db.read().unwrap();

    let mut csv_bytes: Vec<u8> = vec![];
    csv_bytes.extend_from_slice(
      b"timestamp,rank,upvotes,downvotes,reranks,top5_reranks\n",
    );

    db.compress_records(|record| {
      record.timestamp.format_to(&mut csv_bytes).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.data.rank).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.data.upvotes).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.data.downvotes).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.data.reranks).unwrap();
      csv_bytes.push(b',');
      itoa::write(&mut csv_bytes, record.data.top5_reranks).unwrap();
      csv_bytes.push(b'\n');
    });

    let mut res = Response::new(Body::from(csv_bytes));
    res
      .headers_mut()
      .insert(header::CONTENT_TYPE, HeaderValue::from_static("text/csv"));
    Ok(res)
  }
}
