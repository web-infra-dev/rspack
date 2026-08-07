use std::{
  future::pending,
  time::{Duration, Instant},
};

use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use tokio::{
  sync::{mpsc, oneshot},
  time::Instant as TokioInstant,
};

use super::{
  CacheKey, CacheValue, Etag, FileCacheStrategy,
  cache_value::{CacheValueData, ErasedCacheValue},
};

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE: Duration = Duration::from_secs(5);
const DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES: Duration = Duration::from_secs(1);

#[derive(Debug)]
enum Command {
  Store {
    key: CacheKey,
    etag: Option<Etag>,
    value: ErasedCacheValue,
  },
  StoreBuildDependencies(Vec<Utf8PathBuf>),
  Restore {
    key: CacheKey,
    etag: Option<Etag>,
    result: oneshot::Sender<Result<Option<ErasedCacheValue>>>,
  },
  RecordBuildTime(Duration),
  BeginIdle,
  EndIdle,
  Shutdown(oneshot::Sender<Result<()>>),
}

struct BackgroundJob {
  strategy: FileCacheStrategy,
  command_receiver: mpsc::UnboundedReceiver<Command>,
  idle_deadline: Option<TokioInstant>,
  idle_timeout: Duration,
  idle_timeout_for_initial_store: Duration,
  idle_timeout_after_large_changes: Duration,
  time_spent_in_build: Duration,
  avg_time_spent_in_store: Option<Duration>,
}

impl BackgroundJob {
  async fn run(mut self) {
    loop {
      let idle_deadline = self.idle_deadline;

      tokio::select! {
        biased;
        command = self.command_receiver.recv() => {
          let Some(command) = command else {
            if self.strategy.has_pending_writes() {
              tracing::warn!("Idle file cache was dropped before shutdown with pending cache items");
            }
            return;
          };
          if self.handle_command(command).await {
            return;
          }
        }
        _ = wait_for_idle_deadline(idle_deadline) => {
          self.idle_deadline = None;
          self.process_idle_tasks().await;
        }
      }
    }
  }

  async fn handle_command(&mut self, command: Command) -> bool {
    match command {
      Command::Store { key, etag, value } => {
        if let Err(error) = self.strategy.store(key, etag, value).await {
          tracing::warn!("Storing file cache item failed: {error}");
        }
      }
      Command::StoreBuildDependencies(dependencies) => {
        if let Err(error) = self.strategy.store_build_dependencies(dependencies).await {
          tracing::warn!("Storing file cache build dependencies failed: {error}");
        }
      }
      Command::Restore { key, etag, result } => {
        let _ = result.send(self.strategy.restore(key, etag).await);
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
        self.idle_deadline = Some(TokioInstant::now() + timeout);
      }
      Command::EndIdle => {
        self.idle_deadline = None;
      }
      Command::Shutdown(result) => {
        self.idle_deadline = None;
        let _ = result.send(self.shutdown().await);
        return true;
      }
    }
    false
  }

  async fn process_idle_tasks(&mut self) {
    let start = Instant::now();
    if let Err(error) = self.strategy.after_all_stored().await {
      tracing::warn!("Finalizing idle file cache store failed: {error}");
      return;
    }
    let time_spent_in_store = start.elapsed();
    self.avg_time_spent_in_store = Some(
      self
        .avg_time_spent_in_store
        .unwrap_or_default()
        .max(time_spent_in_store)
        .mul_f64(0.9)
        .saturating_add(time_spent_in_store.mul_f64(0.1)),
    );
    self.time_spent_in_build = Duration::ZERO;
  }

  async fn shutdown(&mut self) -> Result<()> {
    self.strategy.after_all_stored().await?;
    self.strategy.shutdown().await
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
        idle_deadline: None,
        idle_timeout,
        idle_timeout_for_initial_store: idle_timeout.min(idle_timeout_for_initial_store),
        idle_timeout_after_large_changes,
        time_spent_in_build: Duration::ZERO,
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

  pub fn store<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>, value: CacheValue<T>) {
    self.send(Command::Store {
      key,
      etag,
      value: value.erase(),
    });
  }

  pub async fn restore<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Result<Option<CacheValue<T>>> {
    Ok(
      self
        .request(|result| Command::Restore { key, etag, result })
        .await?
        .and_then(ErasedCacheValue::downcast),
    )
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
