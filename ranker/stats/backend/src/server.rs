use std::io;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;

use iron::prelude::*;
use iron::typemap::Key;
use iron::StatusCode;
use persistent::State;
use std::net::SocketAddr;

use crate::database::Database;

pub fn start(
  shared_db: Arc<RwLock<Database>>,
) -> impl Future<Item = (), Error = ()> {
  #[derive(Debug, Copy, Clone)]
  struct DatabaseKey;

  impl Key for DatabaseKey {
    type Value = Database;
  }

  fn serve_records(req: &mut Request) -> IronResult<Response> {
    struct ResponseWriter(Arc<RwLock<Database>>);
    impl iron::response::WriteBody for ResponseWriter {
      fn write_body(&mut self, mut writer: &mut dyn Write) -> io::Result<()> {
        let db = self.0.read().unwrap();
        for record in &db.records {
          serde_json::to_writer(&mut writer, &record)?;
          writer.write_all(b"\n")?;
        }
        Ok(())
      }
    }

    let shared_db = req.get::<State<DatabaseKey>>().unwrap();

    let mut res = Response::new();
    res.status = Some(StatusCode::OK);
    res.body = Some(Box::new(ResponseWriter(shared_db)));
    Ok(res)
  }

  let mut chain = Chain::new(serve_records);
  chain.link(State::<DatabaseKey>::both(shared_db));

  let mut server = iron::Iron::new(chain);
  let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
  server.local_address = Some(addr);

  hyper::Server::bind(&addr)
    .tcp_keepalive(server.timeouts.keep_alive)
    .serve(server)
    .map_err(|e| eprintln!("server error: {}", e))
}
