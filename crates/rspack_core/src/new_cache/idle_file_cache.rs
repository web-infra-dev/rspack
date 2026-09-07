use std::{
  sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
  },
  thread,
  time::Duration,
};

use rspack_error::Result;
use rspack_paths::{InternedPathSet, Utf8PathBuf};
use tokio::{
  sync::mpsc,
  time::{Instant, sleep_until},
};

use super::{
  CacheKey, CacheValue, Etag, FileCacheStrategy, Meta,
  cache_value::{CacheValueData, ErasedCacheValue},
};
use crate::{InfrastructureLogger, Logger};

const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(60);
const DEFAULT_IDLE_TIMEOUT_FOR_INITIAL_STORE: Duration = Duration::from_secs(5);
const DEFAULT_IDLE_TIMEOUT_AFTER_LARGE_CHANGES: Duration = Duration::from_secs(1);
const MAX_IDLE_COMPACTION_PASSES: usize = 10;

#[derive(Debug)]
enum Command {
  BeginIdle { epoch: u64, build_time: Duration },
  EndIdle,
  Shutdown,
}

#[derive(Debug, Clone, Copy)]
struct IdleDeadline {
  at: Instant,
  epoch: u64,
}

struct BackgroundJob {
  strategy: Arc<FileCacheStrategy>,
  logger: Arc<InfrastructureLogger>,
  command_receiver: mpsc::UnboundedReceiver<Command>,
  idle_epoch: Arc<AtomicU64>,
  idle_deadline: Option<IdleDeadline>,
  idle_timeout: Duration,
  idle_timeout_for_initial_store: Duration,
  idle_timeout_after_large_changes: Duration,
  time_spent_in_build: Duration,
  avg_time_spent_in_store: Option<Duration>,
}

impl BackgroundJob {
  async fn run(mut self, database_paths: (Utf8PathBuf, Utf8PathBuf)) {
    self.strategy.db_init(database_paths).await;

    let idle_epoch = Arc::clone(&self.idle_epoch);
    let strategy = Arc::clone(&self.strategy);
    loop {
      let idle_deadline = self.idle_deadline;
      let command = tokio::select! {
        biased;
        _ = strategy.wait_until_unavailable() => return,
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
          self.process_idle_tasks(|| idle_epoch.load(Ordering::Acquire) != epoch).await;
          continue;
        }
      };

      let Some(command) = command else {
        if self.strategy.has_pending_writes() {
          self
            .logger
            .warn("Idle file cache was dropped before shutdown with pending cache items");
        }
        return;
      };
      self.handle_command(command).await;
    }
  }

  async fn handle_command(&mut self, command: Command) {
    match command {
      Command::BeginIdle { epoch, build_time } => {
        self.time_spent_in_build = self
          .time_spent_in_build
          .mul_f64(0.9)
          .saturating_add(build_time);
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
      Command::Shutdown => {
        self.idle_deadline = None;
        self.strategy.shutdown().await;
      }
    }
  }

  async fn process_idle_tasks(&mut self, check_idle_ended: impl FnMut() -> bool) {
    let start = Instant::now();
    self
      .strategy
      .after_all_stored(MAX_IDLE_COMPACTION_PASSES, check_idle_ended)
      .await;
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
  strategy: Arc<FileCacheStrategy>,
  logger: Arc<InfrastructureLogger>,
  command_sender: mpsc::UnboundedSender<Command>,
  idle_epoch: Arc<AtomicU64>,
}

impl IdleFileCache {
  pub fn new(
    database_paths: (Utf8PathBuf, Utf8PathBuf),
    strategy: FileCacheStrategy,
    logger: Arc<InfrastructureLogger>,
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
    let strategy = Arc::new(strategy);
    let background_job = BackgroundJob {
      strategy: Arc::clone(&strategy),
      logger: Arc::clone(&logger),
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
        runtime.block_on(background_job.run(database_paths));
      })
      .expect("failed to spawn idle file cache background thread");

    Self {
      strategy,
      logger,
      command_sender,
      idle_epoch,
    }
  }

  fn send(&self, command: Command) {
    if self.strategy.is_available()
      && let Err(e) = self.command_sender.send(command)
    {
      self.logger.warn(format!(
        "Failed to send command to idle file cache background job: {e}"
      ));
    }
  }

  pub fn store<T: CacheValueData>(&self, key: CacheKey, etag: Option<Etag>, value: CacheValue<T>) {
    self
      .strategy
      .store(key, etag, value.erase(), CacheValue::<T>::encoder());
  }

  pub fn restore<T: CacheValueData>(
    &self,
    key: CacheKey,
    etag: Option<Etag>,
  ) -> Option<CacheValue<T>> {
    let restored = self
      .strategy
      .restore(&key, etag.as_ref(), CacheValue::<T>::decoder());
    restored.and_then(ErasedCacheValue::downcast)
  }

  pub fn store_build_dependencies(&self, dependencies: InternedPathSet) {
    self.strategy.store_build_dependencies(dependencies);
  }

  pub fn store_meta(&self, meta: Meta) {
    self.strategy.store_meta(meta);
  }

  pub fn restore_meta(&self) -> Result<Option<Meta>> {
    self.strategy.restore_meta()
  }

  pub fn begin_idle(&self, build_time: Duration) {
    self.send(Command::BeginIdle {
      epoch: self.idle_epoch.load(Ordering::Acquire),
      build_time,
    });
  }

  pub fn end_idle(&self) {
    self.idle_epoch.fetch_add(1, Ordering::Release);
    self.send(Command::EndIdle);
  }

  pub async fn shutdown(&self) {
    self.send(Command::Shutdown);
    self.command_sender.closed().await;
  }
}
