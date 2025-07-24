use std::boxed::Box;

use napi::bindgen_prelude::*;
use napi_derive::*;
use rspack_fs::{FsWatcher, FsWatcherIgnored, FsWatcherOptions, PathUpdater};
use rspack_regex::RspackRegex;

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

#[napi]
pub struct NativeWatcher {
  watcher: FsWatcher,
  closed: bool,
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
      to_fs_watcher_ignored(options.ignored.clone()),
    );

    Self {
      watcher,
      closed: false,
    }
  }

  #[napi]
  #[allow(clippy::too_many_arguments)]
  pub fn watch(
    &mut self,
    reference: Reference<NativeWatcher>,
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    #[napi(ts_arg_type = "(err: Error | null, result: NativeWatchResult) => void")]
    callback: Function<'static>,
    #[napi(ts_arg_type = "(path: string) => void")] callback_undelayed: Function<'static>,
    env: Env,
  ) -> napi::Result<()> {
    if self.closed {
      return Err(napi::Error::from_reason(
        "The native watcher has been closed, cannot watch again.",
      ));
    }

    let js_event_handler = JsEventHandler::new(callback)?;
    let js_event_handler_undelayed = JsEventHandlerUndelayed::new(callback_undelayed)?;

    let file_updater = PathUpdater {
      added: files.0,
      removed: files.1,
    };

    let directories_updater = PathUpdater {
      added: directories.0,
      removed: directories.1,
    };

    let missing_updater = PathUpdater {
      added: missing.0,
      removed: missing.1,
    };

    reference.share_with(env, |native_watcher| {
      napi::bindgen_prelude::spawn(async move {
        native_watcher
          .watcher
          .watch(
            file_updater,
            directories_updater,
            missing_updater,
            Box::new(js_event_handler),
            Box::new(js_event_handler_undelayed),
          )
          .await
      });
      Ok(())
    })?;

    Ok(())
  }

  #[napi]
  /// # Safety
  ///
  /// This function is unsafe because it uses `&mut self` to call the watcher asynchronously.
  /// It's important to ensure that the watcher is not used in any other places before this function is finished.
  /// You must ensure that the watcher not call watch, close or pause in the same time, otherwise it may lead to undefined behavior.
  pub async unsafe fn close(&mut self) -> napi::Result<()> {
    self
      .watcher
      .close()
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    self.closed = true;
    Ok(())
  }

  #[napi]
  pub fn pause(&self) -> napi::Result<()> {
    self
      .watcher
      .pause()
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(())
  }
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

impl rspack_fs::EventAggregateHandler for JsEventHandler {
  fn on_event_handle(
    &self,
    changed_files: std::collections::HashSet<String>,
    deleted_files: std::collections::HashSet<String>,
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
    String,
    napi::Unknown<'static>,
    String,
    Status,
    false,
    false,
    1,
  >,
}

impl JsEventHandlerUndelayed {
  fn new(callback: Function<'static>) -> napi::Result<Self> {
    let callback = callback
      .build_threadsafe_function::<String>()
      .weak::<false>()
      .max_queue_size::<1>()
      .build_callback(
        move |ctx: napi::threadsafe_function::ThreadSafeCallContext<_>| Ok(ctx.value),
      )?;

    Ok(Self { inner: callback })
  }
}

impl rspack_fs::EventHandler for JsEventHandlerUndelayed {
  fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
    self.inner.call(
      changed_file,
      napi::threadsafe_function::ThreadsafeFunctionCallMode::NonBlocking,
    );
    Ok(())
  }
}
