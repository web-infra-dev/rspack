use std::{
  future::pending,
  time::{Duration, Instant},
};

use futures::future::join_all;
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap as HashMap;
use tokio::{
  sync::{mpsc, oneshot},
  time::Instant as TokioInstant,
};

use super::{CacheData, Etag, FileCacheStrategy};

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE: Duration = Duration::from_secs(5);
const DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES: Duration = Duration::from_secs(1);
const MAX_IDLE_TASKS_PER_BATCH: usize = 100;

#[derive(Debug)]
struct PendingStore {
  etag: Option<Etag>,
  data: CacheData,
}

#[derive(Debug)]
enum IdleTask {
  Store {
    identifier: String,
    entry: PendingStore,
  },
  StoreBuildDependencies(Vec<Utf8PathBuf>),
}

#[derive(Debug, Default)]
struct PendingIdleTasks {
  stores: HashMap<String, PendingStore>,
  build_dependencies: Option<Vec<Utf8PathBuf>>,
}

impl PendingIdleTasks {
  fn is_empty(&self) -> bool {
    self.stores.is_empty() && self.build_dependencies.is_none()
  }

  fn take_batch(&mut self) -> Vec<IdleTask> {
    let mut tasks = Vec::with_capacity(MAX_IDLE_TASKS_PER_BATCH);

    while tasks.len() < MAX_IDLE_TASKS_PER_BATCH {
      let Some(identifier) = self.stores.keys().next().cloned() else {
        break;
      };
      let entry = self
        .stores
        .remove(&identifier)
        .expect("pending idle task should exist");
      tasks.push(IdleTask::Store { identifier, entry });
    }

    if tasks.len() < MAX_IDLE_TASKS_PER_BATCH
      && let Some(dependencies) = self.build_dependencies.take()
    {
      tasks.push(IdleTask::StoreBuildDependencies(dependencies));
    }

    tasks
  }
}

#[derive(Debug)]
enum Command {
  Store {
    identifier: String,
    entry: PendingStore,
  },
  StoreBuildDependencies(Vec<Utf8PathBuf>),
  Restore {
    identifier: String,
    etag: Option<Etag>,
    result: oneshot::Sender<Result<Option<CacheData>>>,
  },
  RecordBuildTime(Duration),
  BeginIdle,
  EndIdle,
  Shutdown(oneshot::Sender<Result<()>>),
}

#[derive(Debug, Default, Clone, Copy)]
enum IdleState {
  #[default]
  Active,
  Idle,
  Waiting(TokioInstant),
}

struct BackgroundJob {
  strategy: FileCacheStrategy,
  command_receiver: mpsc::UnboundedReceiver<Command>,
  pending_tasks: PendingIdleTasks,
  idle_state: IdleState,
  idle_timeout: Duration,
  idle_timeout_for_initial_store: Duration,
  idle_timeout_after_large_changes: Duration,
  time_spent_in_build: Duration,
  time_spent_in_store: Duration,
  avg_time_spent_in_store: Option<Duration>,
}

impl BackgroundJob {
  async fn run(mut self) {
    loop {
      let idle_deadline = match self.idle_state {
        IdleState::Waiting(deadline) => Some(deadline),
        IdleState::Active | IdleState::Idle => None,
      };

      tokio::select! {
        biased;
        command = self.command_receiver.recv() => {
          let Some(command) = command else {
            if !self.pending_tasks.is_empty() {
              tracing::warn!("Idle file cache was dropped before shutdown with pending cache items");
            }
            return;
          };
          if self.handle_command(command).await {
            return;
          }
        }
        _ = wait_for_idle_deadline(idle_deadline) => {
          self.idle_state = IdleState::Idle;
          self.process_idle_tasks().await;
        }
      }
    }
  }

  async fn handle_command(&mut self, command: Command) -> bool {
    match command {
      Command::Store { identifier, entry } => {
        self.pending_tasks.stores.insert(identifier, entry);
        self.schedule_pending_tasks();
      }
      Command::StoreBuildDependencies(dependencies) => {
        self.pending_tasks.build_dependencies = Some(dependencies);
        self.schedule_pending_tasks();
      }
      Command::Restore {
        identifier,
        etag,
        result,
      } => {
        let _ = result.send(self.restore(identifier, etag).await);
      }
      Command::RecordBuildTime(build_time) => {
        self.time_spent_in_build = self
          .time_spent_in_build
          .mul_f64(0.9)
          .saturating_add(build_time);
      }
      Command::BeginIdle => {
        let is_initial_store = self.avg_time_spent_in_store.is_none();
        let is_large_change = self.time_spent_in_build
          > self
            .avg_time_spent_in_store
            .unwrap_or_default()
            .saturating_mul(2);
        let mut timeout = self.idle_timeout;
        if is_initial_store {
          timeout = timeout.min(self.idle_timeout_for_initial_store);
        }
        if is_large_change {
          timeout = timeout.min(self.idle_timeout_after_large_changes);
        }
        self.idle_state = IdleState::Waiting(TokioInstant::now() + timeout);
      }
      Command::EndIdle => {
        self.idle_state = IdleState::Active;
      }
      Command::Shutdown(result) => {
        self.idle_state = IdleState::Active;
        let _ = result.send(self.flush_and_clear().await);
        return true;
      }
    }
    false
  }

  fn schedule_pending_tasks(&mut self) {
    if matches!(self.idle_state, IdleState::Idle) {
      self.idle_state = IdleState::Waiting(TokioInstant::now());
    }
  }

  async fn run_pending_tasks(&self, tasks: Vec<IdleTask>) -> Result<()> {
    let strategy = &self.strategy;
    let results = join_all(tasks.into_iter().map(|task| async move {
      match task {
        IdleTask::Store { identifier, entry } => {
          strategy.store(identifier, entry.etag, entry.data).await
        }
        IdleTask::StoreBuildDependencies(dependencies) => {
          strategy.store_build_dependencies(dependencies).await
        }
      }
    }))
    .await;

    for result in results {
      result?;
    }
    Ok(())
  }

  async fn process_idle_tasks(&mut self) {
    let tasks = self.pending_tasks.take_batch();
    if tasks.is_empty() {
      self.finish_idle_cycle().await;
      return;
    }

    let start = Instant::now();
    if let Err(error) = self.run_pending_tasks(tasks).await {
      tracing::warn!("Background tasks during idle failed: {error}");
    }
    self.time_spent_in_store = self.time_spent_in_store.saturating_add(start.elapsed());

    // Return to the command loop between batches. Its biased select gives
    // EndIdle, Restore, and Shutdown priority over more background work.
    if !matches!(self.idle_state, IdleState::Active) {
      self.idle_state = IdleState::Waiting(TokioInstant::now());
    }
  }

  async fn finish_idle_cycle(&mut self) {
    let start = Instant::now();
    if let Err(error) = self.strategy.after_all_stored().await {
      tracing::warn!("Finalizing idle file cache store failed: {error}");
      return;
    }
    self.time_spent_in_store = self.time_spent_in_store.saturating_add(start.elapsed());
    let time_spent_in_store = self.time_spent_in_store;
    self.avg_time_spent_in_store = Some(
      self
        .avg_time_spent_in_store
        .unwrap_or_default()
        .max(time_spent_in_store)
        .mul_f64(0.9)
        .saturating_add(time_spent_in_store.mul_f64(0.1)),
    );
    self.time_spent_in_store = Duration::ZERO;
    self.time_spent_in_build = Duration::ZERO;
  }

  async fn restore(&mut self, identifier: String, etag: Option<Etag>) -> Result<Option<CacheData>> {
    if let Some(pending) = self.pending_tasks.stores.remove(&identifier) {
      let result = self
        .strategy
        .store(identifier.clone(), pending.etag, pending.data)
        .await;
      self.schedule_pending_tasks();
      result?;
    }
    self.strategy.restore(&identifier, etag.as_deref()).await
  }

  async fn flush_and_clear(&mut self) -> Result<()> {
    loop {
      let tasks = self.pending_tasks.take_batch();
      if tasks.is_empty() {
        break;
      }
      self.run_pending_tasks(tasks).await?;
    }
    self.strategy.after_all_stored().await?;
    self.strategy.clear().await
  }
}

async fn wait_for_idle_deadline(idle_deadline: Option<TokioInstant>) {
  match idle_deadline {
    Some(idle_deadline) => tokio::time::sleep_until(idle_deadline).await,
    None => pending().await,
  }
}

/// Runs filesystem cache operations in one persistent background job.
#[derive(Debug)]
pub struct IdleFileCache {
  command_sender: mpsc::UnboundedSender<Command>,
}

impl IdleFileCache {
  pub fn new(strategy: FileCacheStrategy) -> Self {
    Self::with_timeouts(
      strategy,
      DEFAULT_IDLE_TIMEOUT,
      DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE,
      DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES,
    )
  }

  pub fn with_timeouts(
    strategy: FileCacheStrategy,
    idle_timeout: Duration,
    idle_timeout_for_initial_store: Duration,
    idle_timeout_after_large_changes: Duration,
  ) -> Self {
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let _ = rspack_tasks::spawn_in_context(
      BackgroundJob {
        strategy,
        command_receiver,
        pending_tasks: PendingIdleTasks::default(),
        idle_state: IdleState::Active,
        idle_timeout,
        idle_timeout_for_initial_store: idle_timeout.min(idle_timeout_for_initial_store),
        idle_timeout_after_large_changes,
        time_spent_in_build: Duration::ZERO,
        time_spent_in_store: Duration::ZERO,
        avg_time_spent_in_store: None,
      }
      .run(),
    );

    Self { command_sender }
  }

  fn send(&self, command: Command) {
    if self.command_sender.send(command).is_err() {
      tracing::warn!("Idle file cache background job has stopped");
    }
  }

  async fn request<T>(
    &self,
    command: impl FnOnce(oneshot::Sender<Result<T>>) -> Command,
  ) -> Result<T> {
    let (result, result_receiver) = oneshot::channel();
    self
      .command_sender
      .send(command(result))
      .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))?;
    result_receiver
      .await
      .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))?
  }

  pub fn store(&self, identifier: impl Into<String>, etag: Option<Etag>, data: CacheData) {
    self.send(Command::Store {
      identifier: identifier.into(),
      entry: PendingStore { etag, data },
    });
  }

  pub async fn restore(&self, identifier: &str, etag: Option<&str>) -> Result<Option<CacheData>> {
    self
      .request(|result| Command::Restore {
        identifier: identifier.to_string(),
        etag: etag.map(Into::into),
        result,
      })
      .await
  }

  pub fn store_build_dependencies(&self, dependencies: Vec<Utf8PathBuf>) {
    self.send(Command::StoreBuildDependencies(dependencies));
  }

  pub fn record_build_time(&self, build_time: Duration) {
    self.send(Command::RecordBuildTime(build_time));
  }

  pub fn begin_idle(&self) {
    self.send(Command::BeginIdle);
  }

  pub fn end_idle(&self) {
    self.send(Command::EndIdle);
  }

  pub async fn shutdown(&self) -> Result<()> {
    self.request(Command::Shutdown).await
  }
}
