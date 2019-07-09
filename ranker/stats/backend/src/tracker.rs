use failure::{Error, Fail, Fallible};
use log::{debug, info};

use futures::sync::oneshot;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::time::Instant;

use crate::database::Database;
use crate::record::{Record, Timestamp};

type JsonValue = serde_json::Value;

pub fn start(
  shared_db: Arc<RwLock<Database>>,
  shutdown_signal_recv: oneshot::Receiver<()>,
) -> impl Future<Item = (), Error = ()> {
  let http_client = hyper::Client::new();

  let api_url = hyper::Uri::from_static(crate::RANKER_API_URL);

  tokio::timer::Interval::new(Instant::now(), crate::FETCH_INTERVAL)
    .map_err(|e: tokio::timer::Error| Error::from(e.context("timer error")))
    .and_then(move |_: Instant| {
      let timestamp = Timestamp::now();
      info!("sending a request to {}", crate::RANKER_API_URL);

      fetch_json(&http_client, api_url.clone())
        .map_err(|e: Error| Error::from(e.context("API request error")))
        .and_then(move |json: JsonValue| {
          json_to_record(json, timestamp)
            .ok_or_else(|| failure::err_msg("malformed JSON response from API"))
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
    .map_err(|e| crate::print_error(e.as_fail()))
    .select(shutdown_signal_recv.then(|_| {
      debug!("signal received, starting graceful shutdown");
      Ok(())
    }))
    .then(|result| match result {
      Ok(((), _)) => Ok(()),
      Err(((), _)) => Err(()),
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
