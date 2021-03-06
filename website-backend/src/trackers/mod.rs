pub mod ranker;

use failure::{Error, Fail, Fallible};
use log::info;

use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::time::{Duration, Instant};

use crate::database::Database;
use crate::http::HttpClient;
use crate::record::{Record, Timestamp};
use crate::shutdown::Shutdown;

pub trait Tracker {
  type DataPoint;

  fn describe(&self) -> String;

  fn fetch_data_point(
    &self,
    http_client: &HttpClient,
  ) -> Box<dyn Future<Item = Self::DataPoint, Error = Error> + Send>;
}

pub fn start<D: serde::ser::Serialize + std::fmt::Debug>(
  tracker: Box<dyn Tracker<DataPoint = D> + Send>,
  request_interval: Duration,
  shared_db: Arc<RwLock<Database<D>>>,
  shutdown: Shutdown,
) -> impl Future<Item = (), Error = ()> {
  info!("starting {}", tracker.describe());

  let http_client = hyper::Client::new();

  tokio::timer::Interval::new(Instant::now(), request_interval)
    .map_err(|e: tokio::timer::Error| Error::from(e.context("timer error")))
    .and_then(move |_: Instant| {
      let timestamp = Timestamp::now();

      tracker.fetch_data_point(&http_client).then(|r: Result<D, Error>| match r
      {
        Ok(data) => Ok(Some(Record { timestamp, data })),
        Err(e) => {
          log_error!(log::Level::Warn, &e.context("API request error"));
          Ok(None)
        }
      })
    })
    .for_each(move |record: Option<Record<D>>| -> Fallible<()> {
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
