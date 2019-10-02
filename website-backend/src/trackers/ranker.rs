use super::Tracker;
use crate::http::{get_json, HttpClient, JsonValue};
use failure::Error;
use hyper::Uri;
use tokio::prelude::*;

const RANKER_API_URL: &str = "http://api.ranker.com/lists/298553/items/85372114?include=crowdRankedStats,votes";

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DataPoint {
  pub rank: u64,
  pub upvotes: u64,
  pub downvotes: u64,
  pub reranks: u64,
  pub top5_reranks: u64,
}

pub struct RankerTracker {
  url: Uri,
}

impl RankerTracker {
  pub fn new() -> Self {
    Self { url: Uri::from_static(RANKER_API_URL) }
  }
}

impl Tracker for RankerTracker {
  type DataPoint = DataPoint;

  fn describe(&self) -> String {
    "ranker".to_owned()
  }

  fn fetch_data_point(
    &self,
    http_client: &HttpClient,
  ) -> Box<dyn Future<Item = Self::DataPoint, Error = Error> + Send> {
    Box::new(get_json(&http_client, self.url.clone()).and_then(
      |json: JsonValue| {
        json_to_data_point(json)
          .ok_or_else(|| failure::err_msg("malformed JSON response from API"))
      },
    ))
  }
}

fn json_to_data_point(json: JsonValue) -> Option<DataPoint> {
  Some(DataPoint {
    rank: json["rank"].as_u64()?,
    upvotes: json["votes"]["upVotes"].as_u64()?,
    downvotes: json["votes"]["downVotes"].as_u64()?,
    reranks: json["crowdRankedStats"]["totalContributingListCount"].as_u64()?,
    top5_reranks: json["crowdRankedStats"]["top5ListCount"].as_u64()?,
  })
}
