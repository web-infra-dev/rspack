use std::{
  any::Any,
  collections::VecDeque,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
  },
};

use rspack_error::Result;
use rspack_util::ext::AsAny;
use tokio::{
  runtime::Handle,
  sync::mpsc::{self, error::TryRecvError},
};

/// Result returned by task
///
/// The success value will contain the tasks that should run next
pub type TaskResult<Ctx> = Result<Vec<Box<dyn Task<Ctx>>>>;

/// Task type
pub enum TaskType {
  /// Sync Task
  Sync,
  /// Async Task
  Async,
}

/// Used for define tasks
///
/// See test for more example
#[async_trait::async_trait]
pub trait Task<Ctx>: Send + Any + AsAny {
  /// Return the task type
  ///
  /// Return `TaskType::Sync` will run `self::sync_run`
  /// Return `TaskType::Async` will run `self::async_run`
  fn get_task_type(&self) -> TaskType;
  /// get task name for monitor
  fn name(&self) -> &'static str;
  /// Sync task process
  ///
  /// The context is shared with all tasks
  fn sync_run(self: Box<Self>, _context: &mut Ctx) -> TaskResult<Ctx> {
    unreachable!();
  }

  /// Async task process
  async fn async_run(self: Box<Self>) -> TaskResult<Ctx> {
    unreachable!();
  }
}

/// Run task loop
pub fn run_task_loop<Ctx: 'static>(
  ctx: &mut Ctx,
  init_tasks: Vec<Box<dyn Task<Ctx>>>,
) -> Result<()> {
  run_task_loop_with_event(ctx, init_tasks, |_, task| task)
}

/// Run task loop with event
pub fn run_task_loop_with_event<Ctx: 'static>(
  ctx: &mut Ctx,
  init_tasks: Vec<Box<dyn Task<Ctx>>>,
  before_task_run: impl Fn(&mut Ctx, Box<dyn Task<Ctx>>) -> Box<dyn Task<Ctx>>,
) -> Result<()> {
  // create channel to receive async task result
  let (tx, mut rx) = mpsc::unbounded_channel::<TaskResult<Ctx>>();
  // mark whether the task loop has been returned
  // the async task should not call `tx.send` after this mark to true
  let is_expected_shutdown: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
  let (async_task, sync_task): (Vec<_>, Vec<_>) = init_tasks
    .into_iter()
    .partition(|x| matches!(x.get_task_type(), TaskType::Async));
  // FIXME: we should create a TaskQueue struct to abstract sync_task | async_task schedule later
  let mut sync_queue: VecDeque<Box<dyn Task<Ctx>>> = VecDeque::from(sync_task);
  let mut async_queue: VecDeque<Box<dyn Task<Ctx>>> = VecDeque::from(async_task);
  let mut active_task_count = 0;
  tokio::task::block_in_place(|| loop {
    // clear async_loop first, since async_task can run parallel
    while let Some(task) = async_queue.pop_front() {
      let tx = tx.clone();
      let is_expected_shutdown = is_expected_shutdown.clone();
      active_task_count += 1;
      // safe expect here since spawn always return Ok
      tokio::task::Builder::new()
        .name(task.name())
        .spawn(async move {
          let r = task.async_run().await;
          if !is_expected_shutdown.load(Ordering::Relaxed) {
            tx.send(r).expect("failed to send error message");
          }
        })
        .expect("spawn task failed");
    }
    let task = sync_queue.pop_front();
    if task.is_none() && active_task_count == 0 {
      return Ok(());
    }
    if let Some(task) = task {
      let task = before_task_run(ctx, task);
      match task.get_task_type() {
        TaskType::Async => {
          let tx = tx.clone();
          let is_expected_shutdown = is_expected_shutdown.clone();
          active_task_count += 1;
          // safe expect here since spawn always return Ok
          tokio::task::Builder::new()
            .name(task.name())
            .spawn(async move {
              let r = task.async_run().await;
              if !is_expected_shutdown.load(Ordering::Relaxed) {
                tx.send(r).expect("failed to send error message");
              }
            })
            .expect("spawn task failed");
        }
        TaskType::Sync => {
          // merge sync task result directly
          match task.sync_run(ctx) {
            Ok(r) => {
              let (async_task, sync_task): (Vec<_>, Vec<_>) = r
                .into_iter()
                .partition(|x| matches!(x.get_task_type(), TaskType::Async));
              sync_queue.extend(sync_task);
              async_queue.extend(async_task);
            }
            Err(e) => {
              is_expected_shutdown.store(true, Ordering::Relaxed);
              return Err(e);
            }
          }
        }
      }
    }

    let data = if sync_queue.is_empty() && async_queue.is_empty() && active_task_count != 0 {
      Handle::current().block_on(async {
        let res = rx.recv().await.expect("should recv success");
        Ok(res)
      })
    } else {
      rx.try_recv()
    };

    match data {
      Ok(r) => {
        active_task_count -= 1;
        // merge async task result
        match r {
          Ok(r) => {
            let (async_task, sync_task): (Vec<_>, Vec<_>) = r
              .into_iter()
              .partition(|x| matches!(x.get_task_type(), TaskType::Async));
            sync_queue.extend(sync_task);
            async_queue.extend(async_task);
          }
          Err(e) => {
            is_expected_shutdown.store(true, Ordering::Relaxed);
            return Err(e);
          }
        }
      }
      Err(TryRecvError::Empty) => {}
      _ => {
        panic!("unexpected recv error")
      }
    }
  })
}

#[cfg(test)]
mod test {
  use rspack_error::miette::miette;

  use super::*;

  #[derive(Default)]
  struct Context {
    call_sync_task_count: u32,
    max_sync_task_call: u32,
    sync_return_error: bool,
    async_return_error: bool,
  }

  struct SyncTask;
  impl Task<Context> for SyncTask {
    fn name(&self) -> &'static str {
      "sync_task"
    }
    fn get_task_type(&self) -> TaskType {
      TaskType::Sync
    }
    fn sync_run(self: Box<Self>, context: &mut Context) -> TaskResult<Context> {
      if context.sync_return_error {
        return Err(miette!("throw sync error"));
      }

      let async_return_error = context.async_return_error;
      context.call_sync_task_count += 1;
      if context.call_sync_task_count < context.max_sync_task_call {
        return Ok(vec![
          Box::new(AsyncTask { async_return_error }),
          Box::new(AsyncTask { async_return_error }),
        ]);
      }
      Ok(vec![])
    }
  }

  struct AsyncTask {
    async_return_error: bool,
  }
  #[async_trait::async_trait]
  impl Task<Context> for AsyncTask {
    fn name(&self) -> &'static str {
      "async_task"
    }
    fn get_task_type(&self) -> TaskType {
      TaskType::Async
    }
    async fn async_run(self: Box<Self>) -> TaskResult<Context> {
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
      if self.async_return_error {
        Err(miette!("throw async error"))
      } else {
        Ok(vec![Box::new(SyncTask)])
      }
    }
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn test_run_task_loop() {
    let mut context = Context {
      call_sync_task_count: 0,
      max_sync_task_call: 4,
      sync_return_error: false,
      async_return_error: false,
    };
    let res = run_task_loop(
      &mut context,
      vec![Box::new(AsyncTask {
        async_return_error: false,
      })],
    );
    assert!(res.is_ok(), "task loop should be run success");
    assert_eq!(context.call_sync_task_count, 7);

    let mut context = Context {
      call_sync_task_count: 0,
      max_sync_task_call: 4,
      sync_return_error: true,
      async_return_error: false,
    };
    let res = run_task_loop(
      &mut context,
      vec![Box::new(AsyncTask {
        async_return_error: false,
      })],
    );
    assert!(
      format!("{:?}", res).contains("throw sync error"),
      "should return sync error"
    );
    assert_eq!(context.call_sync_task_count, 0);

    let mut context = Context {
      call_sync_task_count: 0,
      max_sync_task_call: 4,
      sync_return_error: false,
      async_return_error: true,
    };
    let res = run_task_loop(
      &mut context,
      vec![Box::new(AsyncTask {
        async_return_error: false,
      })],
    );
    assert!(
      format!("{:?}", res).contains("throw async error"),
      "should return async error"
    );
    assert_eq!(context.call_sync_task_count, 1);
  }
}
