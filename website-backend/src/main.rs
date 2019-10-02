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
mod http;
mod record;
mod server;
mod shutdown;
mod trackers;

use failure::{AsFail, Fail, Fallible, ResultExt};
use log::info;

use futures::sync::oneshot;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;
use tokio_signal::unix::{Signal, SIGINT, SIGTERM};

use std::path::PathBuf;

use crate::config::Config;
use crate::database::Database;
use crate::shutdown::Shutdown;

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
  let db = Database::init(&config.trackers.ranker.database_file)
    .context("failed to initialize database")?;
  let shared_db = Arc::new(RwLock::new(db));

  info!("starting tokio runtime");
  let mut runtime =
    tokio::runtime::Runtime::new().context("failed to start new Runtime")?;

  let shutdown = Shutdown::new();
  let signals_future: oneshot::SpawnHandle<(), ()> =
    oneshot::spawn(receive_signals(shutdown.another()), &runtime.executor());
  let server_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    server::start(config.server, shared_db.clone(), shutdown.another()),
    &runtime.executor(),
  );
  let tracker_future: oneshot::SpawnHandle<(), ()> = oneshot::spawn(
    trackers::start(
      Box::new(trackers::ranker::RankerTracker::new()),
      config.trackers.ranker.request_interval,
      shared_db.clone(),
      shutdown.another(),
    ),
    &runtime.executor(),
  );

  let shutdown_result: Result<(), ()> = runtime
    .block_on(signals_future.join3(server_future, tracker_future).map(|_| ()));
  runtime.shutdown_on_idle().wait().unwrap();
  if shutdown_result.is_err() {
    return Err(failure::err_msg("error in the async code, see logs above"));
  }

  info!("synchronizing database before shutdown");
  let mut db = shared_db.write().unwrap();
  db.write()?;

  Ok(())
}

fn receive_signals(shutdown: Shutdown) -> impl Future<Item = (), Error = ()> {
  let sigint = Signal::new(SIGINT).flatten_stream();
  let sigterm = Signal::new(SIGTERM).flatten_stream();
  Stream::select(sigint, sigterm)
    .into_future()
    .then(|r| match r {
      Ok((Some(s), _)) => {
        info!("received signal {:?}", s);
        Ok(())
      }
      Ok((None, _)) => unreachable!(),
      Err((e, _)) => {
        log_error!(log::Level::Error, e.as_fail());
        Err(())
      }
    })
    .select(shutdown)
    .then(|r| match r {
      Ok(((), _)) => Ok(()),
      Err(((), _)) => Err(()),
    })
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
