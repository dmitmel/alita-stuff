use serde::{Deserialize, Serialize};

pub type Timestamp = i64;

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
  pub timestamp: Timestamp,
  pub rank: u64,
  pub upvotes: u64,
  pub downvotes: u64,
  pub reranks: u64,
  pub top5_reranks: u64,
}

fn format_timestamp(timestamp: Timestamp) -> String {
  let tm: time::Tm = time::at_utc(time::Timespec::new(timestamp, 0));
  format!(
    "{}-{:02}-{:02} {:02}:{:02}:{:02}",
    tm.tm_year + 1900,
    tm.tm_mon + 1,
    tm.tm_mday,
    tm.tm_hour,
    tm.tm_min,
    tm.tm_sec,
  )
}
