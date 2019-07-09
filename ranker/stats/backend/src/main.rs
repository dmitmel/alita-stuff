mod database;
mod record;
mod server;

use failure::{Error, Fail, Fallible};
use log::{debug, error, info};

use futures::sync::oneshot;
use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::time::{Duration, Instant};

use crate::database::Database;
use crate::record::{Record, Timestamp};

const DATABASE_PATH: &str = "database.json";
const RANKER_API_URL: &str = "http://api.ranker.com/lists/298553/items/85372114?include=crowdRankedStats,votes";
const FETCH_INTERVAL: Duration = Duration::from_secs(5 * 60);

type JsonValue = serde_json::Value;

fn main() {
  env_logger::init();

  let db = try_run(|| {
    Database::init(Path::new(DATABASE_PATH)).map_err(|e: Error| {
      Error::from(e.context("database initialization error"))
    })
  });

  let api_url = hyper::Uri::from_static(RANKER_API_URL);

  let mut runtime =
    tokio::runtime::Runtime::new().expect("failed to start new Runtime");

  let http_client = hyper::Client::new();

  let shared_db = Arc::new(RwLock::new(db));

  let (server_shutdown_send, server_shutdown_recv) = oneshot::channel::<()>();
  let server_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    server::start(shared_db.clone(), server_shutdown_recv)
      .map_err(|e| print_error(&e)),
    &runtime.executor(),
  );

  let (tracker_shutdown_send, tracker_shutdown_recv) = oneshot::channel::<()>();
  let tracker_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    tokio::timer::Interval::new(Instant::now(), FETCH_INTERVAL)
      .map_err(|e: tokio::timer::Error| Error::from(e.context("timer error")))
      .and_then(move |_: Instant| {
        let timestamp = Timestamp::now();
        info!("sending a request to {}", RANKER_API_URL);

        fetch_json(&http_client, api_url.clone())
          .map_err(|e: Error| Error::from(e.context("API request error")))
          .and_then(move |json: JsonValue| {
            json_to_record(json, timestamp).ok_or_else(|| {
              failure::err_msg("malformed JSON response from API")
            })
          })
      })
      .for_each(move |record: Record| -> Fallible<()> {
        info!("{:?}", &record);

        let mut db = shared_db.write().unwrap();
        db.push(record).map_err(|e| {
          Error::from(e.context("error when pushing record to the database"))
        })?;

        Ok(())
      })
      .map_err(|e| print_error(&e))
      .select(tracker_shutdown_recv.then(|_| {
        debug!("signal received, starting graceful shutdown");
        Ok(())
      }))
      .then(|result| match result {
        Ok(((), _)) => Ok(()),
        Err(((), _)) => Err(()),
      }),
    &runtime.executor(),
  );

  let shutdown_result: Result<(), ()> = runtime.block_on(
    server_future
      .select(tracker_future)
      .then(|result| {
        if !server_shutdown_send.is_canceled() {
          server_shutdown_send.send(()).unwrap();
        }
        if !tracker_shutdown_send.is_canceled() {
          tracker_shutdown_send.send(()).unwrap();
        }
        result
      })
      .then(|result| -> Box<dyn Future<Item = (), Error = ()> + Send> {
        match result {
          Ok(((), unfinished_future)) => {
            Box::new(unfinished_future.then(move |_| Ok(())))
          }
          Err(((), unfinished_future)) => {
            Box::new(unfinished_future.then(move |_| Err(())))
          }
        }
      }),
  );

  runtime.shutdown_on_idle().wait().unwrap();

  if shutdown_result.is_err() {
    std::process::exit(1);
  }
}

fn fetch_json<I>(
  client: &hyper::Client<hyper::client::HttpConnector>,
  url: hyper::Uri,
) -> impl Future<Item = I, Error = Error>
where
  I: serde::de::DeserializeOwned,
{
  let mut req = hyper::Request::new(hyper::Body::default());
  *req.uri_mut() = url;

  client
    .request(req)
    .and_then(|res| res.into_body().concat2())
    .map_err(|e| e.context("network error").into())
    .and_then(|body| {
      serde_json::from_slice(&body)
        .map_err(|e| e.context("JSON parse error").into())
    })
}

fn json_to_record(json: JsonValue, timestamp: Timestamp) -> Option<Record> {
  Some(Record {
    timestamp,
    rank: json["rank"].as_u64()?,
    upvotes: json["votes"]["upVotes"].as_u64()?,
    downvotes: json["votes"]["downVotes"].as_u64()?,
    reranks: json["crowdRankedStats"]["totalContributingListCount"].as_u64()?,
    top5_reranks: json["crowdRankedStats"]["top5ListCount"].as_u64()?,
  })
}

fn print_record(record: &Record) -> io::Result<()> {
  let stdout = io::stdout();
  let mut stdout = stdout.lock();
  serde_json::to_writer(&mut stdout, &record)?;
  stdout.write_all(b"\n")?;
  stdout.flush()?;
  Ok(())
}

fn try_run<T, F>(f: F) -> T
where
  F: FnOnce() -> Fallible<T>,
{
  f().unwrap_or_else(|e| {
    print_error(&e);
    std::process::exit(1);
  })
}

fn print_error(error: &Error) {
  use std::thread;

  let thread = thread::current();
  let name: &str = thread.name().unwrap_or("<unnamed>");

  error!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    error!("caused by: {}", cause);
  }

  error!("{}", error.backtrace());
  error!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");
}
