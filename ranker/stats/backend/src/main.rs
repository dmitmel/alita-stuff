extern crate failure;
extern crate futures;
extern crate hyper;
extern crate serde;
extern crate serde_json;
extern crate tokio;

mod database;
mod record;
mod server;

use std::io;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use failure::{Error, Fail, Fallible};

use std::time::{Duration, Instant};

use crate::database::Database;
use crate::record::{Record, Timestamp};

const DATABASE_PATH: &str = "database.json";
const RANKER_API_URL: &str = "http://api.ranker.com/lists/298553/items/85372114?include=crowdRankedStats,votes";
const FETCH_INTERVAL: Duration = Duration::from_secs(5 * 60);

type JsonValue = serde_json::Value;

fn main() {
  let db = try_run(|| {
    Database::init(Path::new(DATABASE_PATH)).map_err(|e: Error| {
      Error::from(e.context("database initialization error"))
    })
  });

  let api_url = hyper::Uri::from_static(RANKER_API_URL);

  tokio::run(futures::lazy(move || {
    let http_client = hyper::Client::new();

    let shared_db = Arc::new(RwLock::new(db));

    tokio::spawn(crate::server::start(shared_db.clone()));

    tokio::timer::Interval::new(Instant::now(), FETCH_INTERVAL)
      .map_err(|e: tokio::timer::Error| Error::from(e.context("timer error")))
      .and_then(move |_: Instant| {
        let timestamp: Timestamp = time::get_time().sec;

        fetch_json(&http_client, api_url.clone())
          .map_err(|e: Error| Error::from(e.context("API request error")))
          .and_then(move |json: JsonValue| {
            json_to_record(json, timestamp).ok_or_else(|| {
              failure::err_msg("malformed JSON response from API")
            })
          })
      })
      .for_each(move |record: Record| -> Fallible<()> {
        print_record(&record).map_err(|e| {
          Error::from(e.context("I/O error when printing record"))
        })?;

        let mut db = shared_db.write().unwrap();
        db.push(record).map_err(|e| {
          Error::from(e.context("error when pushing record to the database"))
        })?;
        Ok(())
      })
      .map_err(|e: failure::Error| print_error(e.as_fail()))
  }));
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
    print_error(e.as_fail());
    std::process::exit(1);
  })
}

fn print_error(error: &dyn Fail) {
  use std::thread;

  let thread = thread::current();
  let name: &str = thread.name().unwrap_or("<unnamed>");

  eprintln!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    eprintln!("caused by: {}", cause);
  }

  if let Some(backtrace) = error.backtrace() {
    eprintln!("{}", backtrace);
  }
  eprintln!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");
}
