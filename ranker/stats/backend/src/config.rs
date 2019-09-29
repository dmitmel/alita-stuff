use log::info;

use serde::Deserialize;

use std::fs::File;
use std::io::{self, BufReader};
use std::path::{Path, PathBuf};

use hyper::Uri;
use std::net::SocketAddr;
use std::time::Duration;

#[derive(Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub trackers: TrackersConfig,
}

impl Config {
  pub fn read(path: &Path) -> Result<Self, io::Error> {
    info!("opening file '{}'", path.display());
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_json::from_reader(reader)?;
    Ok(config)
  }
}

#[derive(Deserialize)]
pub struct ServerConfig {
  pub address: SocketAddr,
}

#[derive(Deserialize)]
pub struct TrackersConfig {
  pub ranker: TrackerConfig,
}

#[derive(Deserialize)]
pub struct TrackerConfig {
  #[serde(deserialize_with = "deserialize_seconds")]
  pub request_interval: Duration,
  pub database_file: PathBuf,
}

fn deserialize_seconds<'de, D>(deserializer: D) -> Result<Duration, D::Error>
where
  D: serde::Deserializer<'de>,
{
  let secs = u64::deserialize(deserializer)?;
  Ok(Duration::from_secs(secs))
}

// fn deserialize_uri<'de, D>(deserializer: D) -> Result<Uri, D::Error>
// where
//   D: serde::Deserializer<'de>,
// {
//   let s = String::deserialize(deserializer)?;
//   s.parse().map_err(serde::de::Error::custom)
// }
