use failure::{Error, Fail, Fallible};
use log::info;

use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::time::Instant;

use crate::database::Database;
use crate::record::{Record, Timestamp};
use crate::shutdown::Shutdown;

type JsonValue = serde_json::Value;

pub fn start(
  config: crate::config::TrackerConfig,
  shared_db: Arc<RwLock<Database>>,
  shutdown: Shutdown,
) -> impl Future<Item = (), Error = ()> {
  info!("starting");

  let http_client = hyper::Client::new();

  tokio::timer::Interval::new(Instant::now(), config.request_interval)
    .map_err(|e: tokio::timer::Error| Error::from(e.context("timer error")))
    .and_then(move |_: Instant| {
      let timestamp = Timestamp::now();
      info!("sending a request to '{}'", config.ranker_api_url);

      fetch_json(&http_client, config.ranker_api_url.clone())
        .map_err(|e: Error| Error::from(e.context("API request error")))
        .and_then(move |json: JsonValue| {
          match json_to_record(json, timestamp) {
            Some(record) => Ok(Some(record)),
            None => Err(failure::err_msg("malformed JSON response from API")),
          }
        })
        .or_else(|e: Error| {
          log_error!(log::Level::Warn, e.as_fail());
          Ok(None)
        })
    })
    .for_each(move |record: Option<Record>| -> Fallible<()> {
      if let Some(record) = record {
        info!("{:?}", &record);

        let mut db = shared_db.write().unwrap();
        db.push(record).map_err(|e| {
          Error::from(e.context("failed to push the record to the database"))
        })?;
      }

      Ok(())
    })
    .map_err(|e| log_error!(log::Level::Error, e.as_fail()))
    .select(shutdown)
    .then(|r: Result<((), _), ((), _)>| {
      info!("stopping");
      match r {
        Ok(((), _)) => Ok(()),
        Err(((), _)) => Err(()),
      }
    })
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
