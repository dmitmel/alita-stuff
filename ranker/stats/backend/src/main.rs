macro_rules! log_error {
  ($log_level:expr, $error:expr) => {
    crate::log_error(
      $error,
      $log_level,
      module_path!(),
      module_path!(),
      file!(),
      line!(),
    );
  };
}

mod config;
mod database;
mod record;
mod server;
mod tracker;

use failure::{Fail, Fallible, ResultExt};
use log::info;

use futures::sync::oneshot;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use std::path::PathBuf;

use crate::config::Config;
use crate::database::Database;

fn main() {
  env_logger::init();
  if let Err(e) = run() {
    log_error!(log::Level::Error, e.as_fail());
    std::process::exit(1);
  }
}

fn run() -> Fallible<()> {
  let config_path = std::env::args_os()
    .nth(1)
    .map_or(PathBuf::from("config.json"), PathBuf::from);
  info!("loading config file '{}'", config_path.display());
  let config = Config::read(&config_path).context("failed to load config")?;

  info!("initializing database");
  let db =
    Database::init(config.database).context("failed to initialize database")?;

  info!("starting tokio runtime");
  let mut runtime =
    tokio::runtime::Runtime::new().context("failed to start new Runtime")?;

  let shared_db = Arc::new(RwLock::new(db));

  info!("starting server task");
  let (server_shutdown_send, server_shutdown_recv) = oneshot::channel::<()>();
  let server_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    server::start(config.server, shared_db.clone(), server_shutdown_recv)
      .map_err(|e| log_error!(log::Level::Error, e.as_fail())),
    &runtime.executor(),
  );

  info!("starting tracker task");
  let (tracker_shutdown_send, tracker_shutdown_recv) = oneshot::channel::<()>();
  let tracker_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    tracker::start(config.tracker, shared_db.clone(), tracker_shutdown_recv),
    &runtime.executor(),
  );

  let shutdown_result: Result<(), ()> =
    runtime.block_on(server_future.select(tracker_future).then(
      |result| -> Box<dyn Future<Item = (), Error = ()> + Send> {
        info!("one of the tasks has just exited, starting graceful shutdown");

        if !server_shutdown_send.is_canceled() {
          info!("sending a shutdown signal to the server task");
          server_shutdown_send.send(()).unwrap();
        }

        if !tracker_shutdown_send.is_canceled() {
          info!("sending a shutdown signal to the server task");
          tracker_shutdown_send.send(()).unwrap();
        }

        info!("waiting for the other task to finish before complete shutdown");
        match result {
          Ok(((), unfinished_future)) => {
            Box::new(unfinished_future.then(move |_| Ok(())))
          }
          Err(((), unfinished_future)) => {
            Box::new(unfinished_future.then(move |_| Err(())))
          }
        }
      },
    ));

  runtime.shutdown_on_idle().wait().unwrap();

  shutdown_result
    .map_err(|_| failure::err_msg("error in the async code, see logs above"))
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
    let backtrace_string: String = backtrace.to_string();
    if !backtrace_string.is_empty() {
      __log!("{}", backtrace);
    }
  }
  __log!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");
}
