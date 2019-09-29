use super::{fetch_json, HttpClient, JsonValue, Tracker};
use failure::Error;
use hyper::Uri;
use std::collections::HashMap;
use tokio::prelude::*;

const SUBREDDITS: &[&str] = &["Gunnm", "alitabattleangel"];
const USER_AGENT: &str =
  "subreddit subscriber count tracker v2.0 (by /u/dmitmel)";

#[derive(Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DataPoint {
  pub subscribers: HashMap<String, u64>,
}

pub struct RankerTracker {
  url: Uri,
}

impl Tracker for RankerTracker {
  type DataPoint = DataPoint;

  fn name() -> &'static str {
    "reddit"
  }

  fn new() -> Self {
    Self { url: Uri::from_static(RANKER_API_URL) }
  }

  fn fetch_data_point(
    &self,
    http_client: &HttpClient,
  ) -> Box<dyn Future<Item = Self::DataPoint, Error = Error> + Send> {
    Box::new(fetch_json(&http_client, self.url.clone()).and_then(
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
