use std::{
  boxed::Box,
  panic::AssertUnwindSafe,
  path::{Path, PathBuf},
  sync::Mutex,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures::FutureExt;
use napi::bindgen_prelude::*;
use napi_derive::*;
use rspack_paths::ArcPath;
use rspack_regex::RspackRegex;
use rspack_watcher::{
  EventAggregateHandler, EventHandler, FsEventKind, FsWatcher, FsWatcherControl, FsWatcherIgnored,
  FsWatcherOptions,
};
use tokio::sync::{mpsc, watch};

type JsWatcherIgnored = Either3<String, Vec<String>, RspackRegex>;

fn to_fs_watcher_ignored(ignored: Option<JsWatcherIgnored>) -> FsWatcherIgnored {
  if let Some(ignored) = ignored {
    match ignored {
      Either3::A(path) => FsWatcherIgnored::Path(path),
      Either3::B(paths) => FsWatcherIgnored::Paths(paths),
      Either3::C(regex) => FsWatcherIgnored::Regex(regex),
    }
  } else {
    FsWatcherIgnored::None
  }
}

#[napi(object, object_to_js = false)]
pub struct NativeWatcherOptions {
  pub follow_symlinks: Option<bool>,

  pub poll_interval: Option<u32>,

  pub aggregate_timeout: Option<u32>,

  #[napi(ts_type = "string | string[] | RegExp")]
  /// The ignored paths for the watcher.
  /// It can be a single path, an array of paths, or a regular expression.
  pub ignored: Option<JsWatcherIgnored>,
}

#[napi]
pub struct NativeWatchResult {
  pub changed_files: Vec<String>,
  pub removed_files: Vec<String>,
}

/// A single, undelayed file system event delivered to the `callbackUndelayed`
/// callback. Passed as one object so napi-rs delivers it as a single JS
/// argument unambiguously (a tuple would arrive as an array).
#[napi(object)]
pub struct NativeWatchUndelayedEvent {
  pub kind: String,
  pub path: String,
}

#[napi]
pub struct NativeWatcher {
  command_tx: mpsc::UnboundedSender<NativeWatcherCommand>,
  control: FsWatcherControl,
  close_state_tx: watch::Sender<NativeWatcherCloseState>,
  admission: Mutex<()>,
}

enum NativeWatcherCommand {
  Watch {
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    start_time: SystemTime,
    event_handler: Box<dyn EventAggregateHandler + Send>,
    event_handler_undelayed: Box<dyn EventHandler + Send>,
  },
  Close,
}

#[derive(Clone)]
enum NativeWatcherCloseState {
  Open,
  Closing,
  Closed(std::result::Result<(), String>),
}

const WATCH_AFTER_CLOSE_ERROR: &str = "The native watcher has been closed, cannot watch again.";
const COMMAND_LOOP_STOPPED_ERROR: &str =
  "The native watcher command loop stopped before the request could be processed.";

fn publish_close_result(
  close_state_tx: &watch::Sender<NativeWatcherCloseState>,
  result: std::result::Result<(), String>,
) {
  close_state_tx.send_if_modified(|state| {
    if matches!(state, NativeWatcherCloseState::Closed(_)) {
      return false;
    }

    *state = NativeWatcherCloseState::Closed(result);
    true
  });
}

async fn run_native_watcher_actor(
  mut watcher: FsWatcher,
  command_rx: &mut mpsc::UnboundedReceiver<NativeWatcherCommand>,
) -> std::result::Result<(), String> {
  while let Some(command) = command_rx.recv().await {
    match command {
      NativeWatcherCommand::Watch {
        files,
        directories,
        missing,
        start_time,
        event_handler,
        event_handler_undelayed,
      } => {
        watcher
          .watch(
            to_tuple_path_iterator(files),
            to_tuple_path_iterator(directories),
            to_tuple_path_iterator(missing),
            start_time,
            event_handler,
            event_handler_undelayed,
          )
          .await;
      }
      NativeWatcherCommand::Close => {
        return watcher.close().await.map_err(|error| error.to_string());
      }
    }
  }

  watcher.close().await.map_err(|error| error.to_string())
}

fn spawn_native_watcher_actor(
  watcher: FsWatcher,
  mut command_rx: mpsc::UnboundedReceiver<NativeWatcherCommand>,
  close_state_tx: watch::Sender<NativeWatcherCloseState>,
) {
  rspack_napi::runtime::spawn(async move {
    let result = AssertUnwindSafe(run_native_watcher_actor(watcher, &mut command_rx))
      .catch_unwind()
      .await;
    let result = match result {
      Ok(result) => result,
      Err(payload) => Err(rspack_napi::runtime::panic_to_napi_error(payload).reason),
    };
    publish_close_result(&close_state_tx, result);
  });
}

fn is_open(close_state_tx: &watch::Sender<NativeWatcherCloseState>) -> bool {
  matches!(*close_state_tx.borrow(), NativeWatcherCloseState::Open)
}

fn watch_after_close_error() -> napi::Error {
  napi::Error::from_reason(WATCH_AFTER_CLOSE_ERROR)
}

fn command_loop_stopped_error() -> napi::Error {
  napi::Error::from_reason(COMMAND_LOOP_STOPPED_ERROR)
}

fn admission_error() -> napi::Error {
  napi::Error::from_reason("The native watcher admission lock is poisoned.")
}

async fn wait_for_close(
  mut close_state_rx: watch::Receiver<NativeWatcherCloseState>,
) -> napi::Result<()> {
  loop {
    let result = {
      let state = close_state_rx.borrow_and_update();
      match &*state {
        NativeWatcherCloseState::Closed(result) => Some(result.clone()),
        NativeWatcherCloseState::Open | NativeWatcherCloseState::Closing => None,
      }
    };

    if let Some(result) = result {
      return result.map_err(napi::Error::from_reason);
    }

    close_state_rx.changed().await.map_err(|_| {
      napi::Error::from_reason(
        "The native watcher lifecycle ended before a close result was published.",
      )
    })?;
  }
}

fn timestamp_to_system_time(millis: u64) -> SystemTime {
  UNIX_EPOCH + Duration::from_millis(millis)
}

#[napi]
impl NativeWatcher {
  #[napi(constructor)]
  pub fn new(options: NativeWatcherOptions) -> Self {
    let watcher = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: options.follow_symlinks.unwrap_or(false),
        poll_interval: options.poll_interval,
        aggregate_timeout: options.aggregate_timeout,
      },
      to_fs_watcher_ignored(options.ignored),
    );
    let control = watcher.control();
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let (close_state_tx, _) = watch::channel(NativeWatcherCloseState::Open);
    spawn_native_watcher_actor(watcher, command_rx, close_state_tx.clone());

    Self {
      command_tx,
      control,
      close_state_tx,
      admission: Mutex::new(()),
    }
  }

  #[napi]
  #[allow(clippy::too_many_arguments)]
  pub fn watch(
    &self,
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    start_time: BigInt,
    #[napi(ts_arg_type = "(err: Error | null, result: NativeWatchResult) => void")]
    callback: Function<'static>,
    #[napi(ts_arg_type = "(event: NativeWatchUndelayedEvent) => void")]
    callback_undelayed: Function<'static>,
  ) -> napi::Result<()> {
    if !is_open(&self.close_state_tx) {
      return Err(watch_after_close_error());
    }

    let event_handler = Box::new(JsEventHandler::new(callback)?);
    let event_handler_undelayed = Box::new(JsEventHandlerUndelayed::new(callback_undelayed)?);
    let command = NativeWatcherCommand::Watch {
      files,
      directories,
      missing,
      start_time: timestamp_to_system_time(start_time.get_u64().1),
      event_handler,
      event_handler_undelayed,
    };

    let _admission = self.admission.lock().map_err(|_| admission_error())?;
    if !is_open(&self.close_state_tx) {
      return Err(watch_after_close_error());
    }

    if self.command_tx.send(command).is_err() {
      publish_close_result(
        &self.close_state_tx,
        Err(COMMAND_LOOP_STOPPED_ERROR.to_string()),
      );
      return Err(command_loop_stopped_error());
    }

    Ok(())
  }

  #[napi(ts_type = "(kind: 'change' | 'remove' | 'create', path: string): void")]
  pub fn trigger_event(&self, kind: String, path: String) -> napi::Result<()> {
    let _admission = self.admission.lock().map_err(|_| admission_error())?;
    if !is_open(&self.close_state_tx) {
      return Err(napi::Error::from_reason(
        "The native watcher has been closed, cannot trigger events.",
      ));
    }

    if let Some(kind) = match kind.as_str() {
      "change" => Some(FsEventKind::Change),
      "remove" => Some(FsEventKind::Remove),
      "create" => Some(FsEventKind::Create),
      _ => None,
    } {
      self
        .control
        .trigger_event(&ArcPath::from(AsRef::<Path>::as_ref(&path)), kind);
    }

    Ok(())
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn close<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let close_state_rx = {
      let _admission = self.admission.lock().map_err(|_| admission_error())?;
      let close_state_rx = self.close_state_tx.subscribe();
      let should_send_close = self.close_state_tx.send_if_modified(|state| {
        if matches!(state, NativeWatcherCloseState::Open) {
          *state = NativeWatcherCloseState::Closing;
          true
        } else {
          false
        }
      });

      if should_send_close && self.command_tx.send(NativeWatcherCommand::Close).is_err() {
        publish_close_result(
          &self.close_state_tx,
          Err(COMMAND_LOOP_STOPPED_ERROR.to_string()),
        );
      }

      close_state_rx
    };

    rspack_napi::runtime::promise_from_future(env, wait_for_close(close_state_rx))
  }

  #[napi]
  pub fn pause(&self) -> napi::Result<()> {
    let _admission = self.admission.lock().map_err(|_| admission_error())?;
    if !is_open(&self.close_state_tx) {
      return Err(napi::Error::from_reason(
        "The native watcher has been closed, cannot pause.",
      ));
    }

    self
      .control
      .pause()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(())
  }
}

fn to_tuple_path_iterator(
  tuple: (Vec<String>, Vec<String>),
) -> (impl Iterator<Item = ArcPath>, impl Iterator<Item = ArcPath>) {
  (
    tuple.0.into_iter().map(|s| ArcPath::from(PathBuf::from(s))),
    tuple.1.into_iter().map(|s| ArcPath::from(PathBuf::from(s))),
  )
}

struct JsEventHandler {
  inner: napi::threadsafe_function::ThreadsafeFunction<
    NativeWatchResult,
    napi::Unknown<'static>,
    NativeWatchResult,
    Status,
    true,
    true,
    1,
  >,
}

impl JsEventHandler {
  fn new(callback: Function<'static>) -> napi::Result<Self> {
    let callback = callback
      .build_threadsafe_function::<NativeWatchResult>()
      .callee_handled::<true>()
      .max_queue_size::<1>()
      .weak::<true>()
      .build_callback(
        move |ctx: napi::threadsafe_function::ThreadSafeCallContext<_>| Ok(ctx.value),
      )?;

    Ok(Self { inner: callback })
  }
}

impl rspack_watcher::EventAggregateHandler for JsEventHandler {
  fn on_event_handle(
    &self,
    changed_files: rspack_util::fx_hash::FxHashSet<String>,
    deleted_files: rspack_util::fx_hash::FxHashSet<String>,
  ) {
    let changed_files_vec: Vec<String> = changed_files.into_iter().collect();
    let deleted_files_vec: Vec<String> = deleted_files.into_iter().collect();
    let result = NativeWatchResult {
      changed_files: changed_files_vec,
      removed_files: deleted_files_vec,
    };
    self.inner.call(
      Ok(result),
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
  }

  fn on_error(&self, error: rspack_error::Error) {
    // Handle error, maybe log it or notify the user
    let error_message = format!("Watcher error: {error}");
    self.inner.call(
      Err(napi::Error::from_reason(error_message)),
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
  }
}

struct JsEventHandlerUndelayed {
  inner: napi::threadsafe_function::ThreadsafeFunction<
    NativeWatchUndelayedEvent,
    napi::Unknown<'static>,
    NativeWatchUndelayedEvent,
    Status,
    false,
    false,
    1,
  >,
}

impl JsEventHandlerUndelayed {
  fn new(callback: Function<'static>) -> napi::Result<Self> {
    let callback = callback
      .build_threadsafe_function::<NativeWatchUndelayedEvent>()
      .weak::<false>()
      .max_queue_size::<1>()
      .build_callback(
        move |ctx: napi::threadsafe_function::ThreadSafeCallContext<_>| Ok(ctx.value),
      )?;

    Ok(Self { inner: callback })
  }
}

impl rspack_watcher::EventHandler for JsEventHandlerUndelayed {
  fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
    self.inner.call(
      NativeWatchUndelayedEvent {
        kind: "change".to_string(),
        path: changed_file,
      },
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
    Ok(())
  }

  fn on_delete(&self, deleted_file: String) -> rspack_error::Result<()> {
    self.inner.call(
      NativeWatchUndelayedEvent {
        kind: "remove".to_string(),
        path: deleted_file,
      },
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
    Ok(())
  }
}
