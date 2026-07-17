use std::{
  boxed::Box,
  path::PathBuf,
  time::{Duration, SystemTime, UNIX_EPOCH},
};

use napi::bindgen_prelude::*;
use napi_derive::*;
use rspack_paths::ArcPath;
use rspack_regex::RspackRegex;
use rspack_watcher::{
  FsEventKind, FsWatcherHandle, FsWatcherHandleError, FsWatcherIgnored, FsWatcherOperation,
  FsWatcherOptions,
};

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
  watcher: FsWatcherHandle,
}

fn to_napi_error(error: FsWatcherHandleError) -> napi::Error {
  let reason = match error {
    FsWatcherHandleError::Closed(FsWatcherOperation::Watch) => {
      "The native watcher has been closed, cannot watch again.".to_string()
    }
    FsWatcherHandleError::Closed(FsWatcherOperation::TriggerEvent) => {
      "The native watcher has been closed, cannot trigger events.".to_string()
    }
    FsWatcherHandleError::Closed(FsWatcherOperation::Pause) => {
      "The native watcher has been closed, cannot pause.".to_string()
    }
    FsWatcherHandleError::Unavailable => "The native watcher is unavailable.".to_string(),
    FsWatcherHandleError::Internal(error) => error.to_string(),
  };

  napi::Error::from_reason(reason)
}

fn timestamp_to_system_time(millis: u64) -> SystemTime {
  UNIX_EPOCH + Duration::from_millis(millis)
}

#[napi]
impl NativeWatcher {
  #[napi(constructor)]
  pub fn new(options: NativeWatcherOptions) -> Self {
    let (watcher, task) = FsWatcherHandle::new(
      FsWatcherOptions {
        follow_symlinks: options.follow_symlinks.unwrap_or(false),
        poll_interval: options.poll_interval,
        aggregate_timeout: options.aggregate_timeout,
      },
      to_fs_watcher_ignored(options.ignored),
    );
    rspack_napi::runtime::spawn(task);

    Self { watcher }
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
    let event_handler = Box::new(JsEventHandler::new(callback)?);
    let event_handler_undelayed = Box::new(JsEventHandlerUndelayed::new(callback_undelayed)?);

    self
      .watcher
      .watch(
        to_tuple_paths(files),
        to_tuple_paths(directories),
        to_tuple_paths(missing),
        timestamp_to_system_time(start_time.get_u64().1),
        event_handler,
        event_handler_undelayed,
      )
      .map_err(to_napi_error)
  }

  #[napi(ts_type = "(kind: 'change' | 'remove' | 'create', path: string): void")]
  pub fn trigger_event(&self, kind: String, path: String) -> napi::Result<()> {
    let Some(kind) = (match kind.as_str() {
      "change" => Some(FsEventKind::Change),
      "remove" => Some(FsEventKind::Remove),
      "create" => Some(FsEventKind::Create),
      _ => None,
    }) else {
      return Ok(());
    };

    self
      .watcher
      .trigger_event(&ArcPath::from(PathBuf::from(path)), kind)
      .map_err(to_napi_error)
  }

  #[napi(ts_return_type = "Promise<void>")]
  pub fn close<'env>(&self, env: &'env Env) -> napi::Result<PromiseRaw<'env, ()>> {
    let close = self.watcher.close();

    rspack_napi::runtime::promise_from_future(
      env,
      async move { close.await.map_err(to_napi_error) },
    )
  }

  #[napi]
  pub fn pause(&self) -> napi::Result<()> {
    self.watcher.pause().map_err(to_napi_error)
  }
}

fn to_tuple_paths(tuple: (Vec<String>, Vec<String>)) -> (Vec<ArcPath>, Vec<ArcPath>) {
  (
    tuple
      .0
      .into_iter()
      .map(|path| ArcPath::from(PathBuf::from(path)))
      .collect(),
    tuple
      .1
      .into_iter()
      .map(|path| ArcPath::from(PathBuf::from(path)))
      .collect(),
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
