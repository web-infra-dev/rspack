use std::{
  any::Any,
  collections::VecDeque,
  fmt::Debug,
  sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
  },
};

use rspack_error::Result;
use rspack_util::ext::AsAny;
use tokio::{
  sync::mpsc::{self, UnboundedReceiver, UnboundedSender, error::TryRecvError},
  task,
};
use tracing::Instrument;

/// Result returned by task
///
/// The success value will contain the tasks that should run next
pub type TaskResult<Ctx> = Result<Vec<Box<dyn Task<Ctx>>>>;

/// Task type
pub enum TaskType {
  /// Main Task
  Main,
  /// Background Task
  Background,
}

/// Used for define tasks
///
/// See test for more example
#[async_trait::async_trait]
pub trait Task<Ctx>: Debug + Send + Any + AsAny {
  /// Return the task type
  ///
  /// Return `TaskType::Main` will run `self::main_run`
  /// Return `TaskType::Background` will run `self::background_run`
  fn get_task_type(&self) -> TaskType;

  /// can be running in main thread
  async fn main_run(self: Box<Self>, _context: &mut Ctx) -> TaskResult<Ctx> {
    unreachable!();
  }

  /// can be running in background thread
  async fn background_run(self: Box<Self>) -> TaskResult<Ctx> {
    unreachable!();
  }
}

struct TaskLoop<Ctx> {
  /// Main tasks run sequentially in the queue
  main_task_queue: VecDeque<Box<dyn Task<Ctx>>>,
  /// The count of the running background tasks which run immediately in tokio thread workers when they are returned
  background_task_count: u32,
  /// Mark whether the task loop has been returned.
  /// The async task should not call `tx.send` after this mark to true
  is_expected_shutdown: Arc<AtomicBool>,
  /// Used for sending async task results in background tasks
  task_result_sender: UnboundedSender<TaskResult<Ctx>>,
  /// Used for receiving async task results
  task_result_receiver: UnboundedReceiver<TaskResult<Ctx>>,
}

impl<Ctx: 'static> TaskLoop<Ctx> {
  fn new(init_main_tasks: Vec<Box<dyn Task<Ctx>>>) -> Self {
    let (tx, rx) = mpsc::unbounded_channel::<TaskResult<Ctx>>();
    Self {
      main_task_queue: VecDeque::from(init_main_tasks),
      is_expected_shutdown: Arc::new(AtomicBool::new(false)),
      background_task_count: 0,
      task_result_sender: tx,
      task_result_receiver: rx,
    }
  }

  async fn run_task_loop(
    &mut self,
    ctx: &mut Ctx,
    init_background_tasks: Vec<Box<dyn Task<Ctx>>>,
  ) -> Result<()> {
    for background_task in init_background_tasks {
      self.spawn_background(background_task);
    }

    loop {
      let task = self.main_task_queue.pop_front();

      // If there's no main tasks and background tasksm
      if task.is_none() && self.background_task_count == 0 {
        return Ok(());
      }

      // Background tasks are launched as soon as they are returned, so we don't put them into the queue.
      if let Some(task) = task {
        debug_assert!(matches!(task.get_task_type(), TaskType::Main));
        self.handle_task_result(task.main_run(ctx).await)?;
      }

      let data = if self.main_task_queue.is_empty() && self.background_task_count != 0 {
        let res = self
          .task_result_receiver
          .recv()
          .await
          .expect("should recv success");
        Ok(res)
      } else {
        self.task_result_receiver.try_recv()
      };

      match data {
        Ok(r) => {
          self.background_task_count -= 1;
          self.handle_task_result(r)?;
        }
        Err(TryRecvError::Empty) => {}
        _ => {
          panic!("unexpected recv error")
        }
      }
    }
  }

  /// Merge sync task result directly
  fn handle_task_result(&mut self, result: TaskResult<Ctx>) -> Result<()> {
    match result {
      Ok(tasks) => {
        for task in tasks {
          match task.get_task_type() {
            TaskType::Main => self.main_task_queue.push_back(task),
            TaskType::Background => self.spawn_background(task),
          }
        }
        Ok(())
      }
      Err(e) => {
        self.is_expected_shutdown.store(true, Ordering::Relaxed);
        Err(e)
      }
    }
  }

  fn spawn_background(&mut self, task: Box<dyn Task<Ctx>>) {
    let tx = self.task_result_sender.clone();
    let is_expected_shutdown = self.is_expected_shutdown.clone();
    self.background_task_count += 1;
    rspack_tasks::spawn_in_compiler_context(task::unconstrained(
      async move {
        let r = task.background_run().await;
        if !is_expected_shutdown.load(Ordering::Relaxed) {
          tx.send(r).expect("failed to send task result");
        }
      }
      .in_current_span(),
    ));
  }
}

pub async fn run_task_loop<Ctx: 'static>(
  ctx: &mut Ctx,
  init_tasks: Vec<Box<dyn Task<Ctx>>>,
) -> Result<()> {
  let (background_tasks, main_tasks) = init_tasks
    .into_iter()
    .partition(|task| matches!(task.get_task_type(), TaskType::Background));
  let mut task_loop = TaskLoop::new(main_tasks);
  task_loop.run_task_loop(ctx, background_tasks).await
}

#[cfg(test)]
mod test {
  use rspack_error::error;
  use rspack_tasks::within_compiler_context_for_testing;

  use super::*;

  #[derive(Default)]
  struct Context {
    call_sync_task_count: u32,
    max_sync_task_call: u32,
    sync_return_error: bool,
    async_return_error: bool,
  }

  #[derive(Debug)]
  struct SyncTask;
  #[async_trait::async_trait]
  impl Task<Context> for SyncTask {
    fn get_task_type(&self) -> TaskType {
      TaskType::Main
    }
    async fn main_run(self: Box<Self>, context: &mut Context) -> TaskResult<Context> {
      if context.sync_return_error {
        return Err(error!("throw sync error"));
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

  #[derive(Debug)]
  struct AsyncTask {
    async_return_error: bool,
  }
  #[async_trait::async_trait]
  impl Task<Context> for AsyncTask {
    fn get_task_type(&self) -> TaskType {
      TaskType::Background
    }
    async fn background_run(self: Box<Self>) -> TaskResult<Context> {
      tokio::time::sleep(std::time::Duration::from_millis(10)).await;
      if self.async_return_error {
        Err(error!("throw async error"))
      } else {
        Ok(vec![Box::new(SyncTask)])
      }
    }
  }

  #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
  async fn test_run_task_loop() {
    within_compiler_context_for_testing(async {
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
      )
      .await;
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
      )
      .await;
      assert!(
        format!("{res:?}").contains("throw sync error"),
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
      )
      .await;
      assert!(
        format!("{res:?}").contains("throw async error"),
        "should return async error"
      );
      assert_eq!(context.call_sync_task_count, 1);
    })
    .await;
  }
}
