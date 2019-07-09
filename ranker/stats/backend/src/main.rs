mod database;
mod record;
mod server;
mod tracker;

use failure::{Error, Fail};
use log::{error, info};

use futures::sync::oneshot;
use std::path::Path;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::time::Duration;

use crate::database::Database;

const DATABASE_PATH: &str = "database.json";
const RANKER_API_URL: &str = "http://api.ranker.com/lists/298553/items/85372114?include=crowdRankedStats,votes";
const FETCH_INTERVAL: Duration = Duration::from_secs(5 * 60);

fn main() {
  env_logger::init();
  if let Err(()) = run() {
    std::process::exit(1);
  }
}

fn run() -> Result<(), ()> {
  info!("initializing database");
  let db = Database::init(Path::new(DATABASE_PATH))
    .map_err(|e: Error| print_error(e.as_fail()))?;

  let mut runtime =
    tokio::runtime::Runtime::new().expect("failed to start new Runtime");

  let shared_db = Arc::new(RwLock::new(db));

  let (server_shutdown_send, server_shutdown_recv) = oneshot::channel::<()>();
  let server_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    server::start(shared_db.clone(), server_shutdown_recv)
      .map_err(|e| print_error(e.as_fail())),
    &runtime.executor(),
  );

  let (tracker_shutdown_send, tracker_shutdown_recv) = oneshot::channel::<()>();
  let tracker_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    tracker::start(shared_db.clone(), tracker_shutdown_recv),
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

  shutdown_result
}

fn print_error(error: &dyn Fail) {
  use std::thread;

  let thread = thread::current();
  let name: &str = thread.name().unwrap_or("<unnamed>");

  error!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    error!("caused by: {}", cause);
  }

  if let Some(backtrace) = error.backtrace() {
    error!("{}", backtrace);
  }
  error!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");
}
