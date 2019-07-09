macro_rules! log_error {
  ($log_level:expr, $error:expr) => {
    crate::log_error($error, $log_level, module_path!(), module_path!(), file!(), line!());
  };
}

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
    .map_err(|e: Error| log_error!(log::Level::Error, e.as_fail()))?;

  let mut runtime =
    tokio::runtime::Runtime::new().expect("failed to start new Runtime");

  let shared_db = Arc::new(RwLock::new(db));

  let (server_shutdown_send, server_shutdown_recv) = oneshot::channel::<()>();
  let server_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    server::start(shared_db.clone(), server_shutdown_recv)
      .map_err(|e| log_error!(log::Level::Error, e.as_fail())),
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

fn log_error(
  error: &dyn Fail,
  log_level: log::Level,
  log_target: &str,
  log_module_path: &str,
  log_file: &str,
  log_line: u32,
) {
  let thread = std::thread::current();
  let name: &str = thread.name().unwrap_or("<unnamed>");

  macro_rules! __log {
    ($($arg:tt)+) => ({
      log::__private_api_log(
        format_args!($($arg)+),
        log_level,
        &(log_target, log_module_path, log_file, log_line),
      );
    });
  }

  __log!("error in thread '{}': {}", name, error);

  for cause in error.iter_causes() {
    __log!("caused by: {}", cause);
  }

  if let Some(backtrace) = error.backtrace() {
    __log!("{}", backtrace);
  }
  __log!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");
}
