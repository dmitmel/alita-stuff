use failure::{Error, Fail};
use log::info;

use hyper::{Body, Chunk, Request, Uri};
use tokio::prelude::*;

pub type JsonValue = serde_json::Value;

pub type HttpClient = hyper::Client<hyper::client::HttpConnector>;

pub fn get_json<I>(
  client: &HttpClient,
  url: Uri,
) -> impl Future<Item = I, Error = Error>
where
  I: serde::de::DeserializeOwned,
{
  let mut req = Request::new(Body::default());
  *req.uri_mut() = url;
  request(client, req).map_err(|e| e.context("network error").into()).and_then(
    |body| {
      serde_json::from_slice(&body)
        .map_err(|e| e.context("JSON parse error").into())
    },
  )
}

pub fn request(
  client: &HttpClient,
  req: Request<Body>,
) -> impl Future<Item = Chunk, Error = hyper::Error> {
  info!("sending a request to '{}'", req.uri());
  client.request(req).and_then(|res| res.into_body().concat2())
}
