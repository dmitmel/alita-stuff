use futures::task::Task;
use futures::{Async, Future, Poll};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub struct Shutdown {
  id: usize,
  inner: Arc<(AtomicBool, Mutex<Vec<Option<Task>>>)>,
}

impl Shutdown {
  pub fn new() -> Self {
    Self {
      id: 0,
      inner: Arc::new({
        let ready = AtomicBool::new(false);
        let tasks = Mutex::new(vec![None]);
        (ready, tasks)
      }),
    }
  }

  pub fn another(&self) -> Self {
    let inner = self.inner.clone();
    Self {
      id: {
        let (_, tasks) = &*inner;
        let mut tasks = tasks.lock().unwrap();
        let id = tasks.len();
        tasks.push(None);
        id
      },
      inner,
    }
  }
}

impl Drop for Shutdown {
  fn drop(&mut self) {
    let (ready, tasks) = &*self.inner;
    let prev_ready = ready.swap(true, Ordering::SeqCst);
    if !prev_ready {
      for (index, task) in tasks.lock().unwrap().iter().enumerate() {
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
    let (ready, tasks) = &*self.inner;
    if ready.load(Ordering::SeqCst) {
      Ok(Async::Ready(()))
    } else {
      let task = futures::task::current();
      tasks.lock().unwrap()[self.id] = Some(task);
      Ok(Async::NotReady)
    }
  }
}
