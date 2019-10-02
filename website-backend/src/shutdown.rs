use futures::task::Task;
use futures::{Async, Future, Poll};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Shutdown {
  id: usize,
  shared: Arc<Mutex<Shared>>,
}

#[derive(Debug)]
struct Shared {
  ready: bool,
  tasks: Vec<Option<Task>>,
}

impl Shutdown {
  pub fn new() -> Self {
    Self {
      id: 0,
      shared: Arc::new(Mutex::new(Shared { ready: false, tasks: vec![None] })),
    }
  }

  pub fn another(&self) -> Self {
    let shared = self.shared.clone();
    Self {
      id: {
        let mut shared = shared.lock().unwrap();
        let id = shared.tasks.len();
        shared.tasks.push(None);
        id
      },
      shared,
    }
  }
}

impl Drop for Shutdown {
  fn drop(&mut self) {
    let mut shared = self.shared.lock().unwrap();
    if !shared.ready {
      shared.ready = true;
      for (index, task) in shared.tasks.iter().enumerate() {
        if index != self.id {
          if let Some(task) = task {
            task.notify();
          }
        }
      }
    }
  }
}

impl Future for Shutdown {
  type Item = ();
  type Error = ();

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    let mut shared = self.shared.lock().unwrap();
    if shared.ready {
      Ok(Async::Ready(()))
    } else {
      let task = futures::task::current();
      shared.tasks[self.id] = Some(task);
      Ok(Async::NotReady)
    }
  }
}
