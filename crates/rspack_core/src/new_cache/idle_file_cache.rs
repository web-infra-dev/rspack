use std::{
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
    mpsc as sync_mpsc,
  },
  thread,
  time::Duration,
};

use rspack_error::Result;
use rspack_paths::InternedPathSet;
use tokio::{
  sync::{mpsc, oneshot},
  time::{Instant, sleep_until},
};

use super::{
  CacheKey, CacheValue, Etag, FileCacheStrategy,
  cache_value::{CacheValueData, CacheValueDecoder, CacheValueEncoder, ErasedCacheValue},
};

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE: Duration = Duration::from_secs(5);
const DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES: Duration = Duration::from_secs(1);
const MAX_IDLE_COMPACTION_PASSES: usize = 10;

#[derive(Debug)]
enum Command {
  Store {
    key: CacheKey,
    etag: Option<Etag>,
    value: ErasedCacheValue,
    encoder: CacheValueEncoder,
  },
  StoreBuildDependencies(InternedPathSet),
  StoreDependencyId(u32),
  Restore {
    key: CacheKey,
    etag: Option<Etag>,
    decoder: CacheValueDecoder,
    result: sync_mpsc::SyncSender<Result<Option<ErasedCacheValue>>>,
  },
  RecordBuildTime(Duration),
  BeginIdle {
    epoch: u64,
  },
  EndIdle,
  Shutdown(oneshot::Sender<Result<()>>),
  RestoreDependencyId {
    result: sync_mpsc::SyncSender<Result<Option<u32>>>,
  },
}

#[derive(Debug, Clone, Copy)]
struct IdleDeadline {
  at: Instant,
  epoch: u64,
}

struct BackgroundJob {
  strategy: FileCacheStrategy,
  command_receiver: mpsc::UnboundedReceiver<Command>,
  // A deadline remains valid only while this still matches its captured epoch.
  idle_epoch: Arc<AtomicU64>,
  idle_deadline: Option<IdleDeadline>,
  idle_timeout: Duration,
  idle_timeout_for_initial_store: Duration,
  idle_timeout_after_large_changes: Duration,
  time_spent_in_build: Duration,
  avg_time_spent_in_store: Option<Duration>,
}

impl BackgroundJob {
  async fn run(mut self) {
    if let Err(error) = self.strategy.db_validation().await {
      tracing::warn!("Validating persistent cache build dependencies failed: {error}");
      return;
    }

    let idle_epoch = Arc::clone(&self.idle_epoch);
    loop {
      let idle_deadline = self.idle_deadline;
      let command = tokio::select! {
        biased;
        command = self.command_receiver.recv() => command,
        epoch = async {
          match idle_deadline {
            Some(deadline) => {
              sleep_until(deadline.at).await;
              deadline.epoch
            },
            None => std::future::pending().await,
          }
        } => {
          self.idle_deadline = None;
          self
            .process_idle_tasks(|| idle_epoch.load(Ordering::Acquire) != epoch)
            .await;
          continue;
        }
      };

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
  }

  async fn handle_command(&mut self, command: Command) -> bool {
    match command {
      Command::Store {
        key,
        etag,
        value,
        encoder,
      } => {
        self.strategy.store(key, etag, value, encoder);
      }
      Command::StoreBuildDependencies(dependencies) => {
        self.strategy.store_build_dependencies(dependencies);
      }
      Command::StoreDependencyId(dependency_id) => {
        self.strategy.store_dependency_id(dependency_id);
      }
      Command::Restore {
        key,
        etag,
        decoder,
        result,
      } => {
        let _ = result.send(self.strategy.restore(&key, etag.as_ref(), decoder));
      }
      Command::RecordBuildTime(build_time) => {
        self.time_spent_in_build = self
          .time_spent_in_build
          .mul_f64(0.9)
          .saturating_add(build_time);
      }
      Command::BeginIdle { epoch } => {
        if self.idle_epoch.load(Ordering::Acquire) == epoch {
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
          self.idle_deadline = Some(IdleDeadline {
            at: Instant::now() + timeout,
            epoch,
          });
        }
      }
      Command::EndIdle => {
        self.idle_deadline = None;
      }
      Command::Shutdown(result) => {
        self.idle_deadline = None;
        let _ = result.send(self.strategy.shutdown().await);
        return true;
      }
      Command::RestoreDependencyId { result } => {
        let _ = result.send(self.strategy.restore_dependency_id());
      }
    }
    false
  }

  async fn process_idle_tasks(&mut self, check_idle_ended: impl FnMut() -> bool) {
    let start = Instant::now();
    if let Err(error) = self
      .strategy
      .after_all_stored(MAX_IDLE_COMPACTION_PASSES, check_idle_ended)
      .await
    {
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
}

/// Runs filesystem cache operations in one persistent background job.
#[derive(Debug)]
pub struct IdleFileCache {
  command_sender: mpsc::UnboundedSender<Command>,
  idle_epoch: Arc<AtomicU64>,
}

impl IdleFileCache {
  pub fn new(
    strategy: FileCacheStrategy,
    idle_timeout: Option<Duration>,
    idle_timeout_for_initial_store: Option<Duration>,
    idle_timeout_after_large_changes: Option<Duration>,
  ) -> Self {
    let idle_timeout = idle_timeout.unwrap_or(DEFAULT_IDLE_TIMEOUT);
    let idle_timeout_for_initial_store =
      idle_timeout_for_initial_store.unwrap_or(DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE);
    let idle_timeout_after_large_changes =
      idle_timeout_after_large_changes.unwrap_or(DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES);
    let (command_sender, command_receiver) = mpsc::unbounded_channel();
    let idle_epoch = Arc::new(AtomicU64::new(0));
    let background_job = BackgroundJob {
      strategy,
      command_receiver,
      idle_epoch: Arc::clone(&idle_epoch),
      idle_deadline: None,
      idle_timeout,
      idle_timeout_for_initial_store,
      idle_timeout_after_large_changes,
      time_spent_in_build: Duration::ZERO,
      avg_time_spent_in_store: None,
    };
    let _ = thread::Builder::new()
      .name("rspack-idle-file-cache".to_string())
      .spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
          .enable_time()
          .build()
          .expect("failed to create idle file cache runtime");
        runtime.block_on(background_job.run());
      })
      .expect("failed to spawn idle file cache background thread");

    Self {
      command_sender,
      idle_epoch,
    }
  }

  fn send(&self, command: Command) -> Result<()> {
    self
      .command_sender
      .send(command)
      .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))
  }

  pub fn store<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
    value: CacheValue<T>,
  ) -> Result<()> {
    self.send(Command::Store {
      key,
      etag,
      value: value.erase(),
      encoder: CacheValue::<T>::encoder(),
    })
  }

  pub fn restore<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Result<Option<CacheValue<T>>> {
    let (result, result_receiver) = sync_mpsc::sync_channel(1);
    self.send(Command::Restore {
      key,
      etag,
      decoder: CacheValue::<T>::decoder(),
      result,
    })?;
    Ok(
      result_receiver
        .recv()
        .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))??
        .and_then(ErasedCacheValue::downcast),
    )
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) -> Result<()> {
    self.send(Command::StoreBuildDependencies(dependencies))
  }

  pub fn store_dependency_id(&self, dependency_id: u32) -> Result<()> {
    self.send(Command::StoreDependencyId(dependency_id))
  }

  pub fn restore_dependency_id(&self) -> Result<Option<u32>> {
    let (result, result_receiver) = sync_mpsc::sync_channel(1);
    self.send(Command::RestoreDependencyId { result })?;
    result_receiver
      .recv()
      .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))?
  }

  pub fn record_build_time(&self, build_time: Duration) -> Result<()> {
    self.send(Command::RecordBuildTime(build_time))
  }

  pub fn begin_idle(&self) -> Result<()> {
    self.send(Command::BeginIdle {
      epoch: self.idle_epoch.load(Ordering::Acquire),
    })
  }

  pub fn end_idle(&self) -> Result<()> {
    self.idle_epoch.fetch_add(1, Ordering::Release);
    self.send(Command::EndIdle)
  }

  pub async fn shutdown(&self) -> Result<()> {
    let (result, result_receiver) = oneshot::channel();
    self.send(Command::Shutdown(result))?;
    result_receiver
      .await
      .map_err(|_| rspack_error::error!("Idle file cache background job has stopped"))?
  }
}
