use std::boxed::Box;

use async_trait::async_trait;
use napi::bindgen_prelude::{FnArgs, Promise};
use napi_derive::*;
use rspack_fs::{FsWatcher, FsWatcherOptions, Ignored, PathUpdater};
use rspack_napi::threadsafe_function::ThreadsafeFunction;

struct SafetyIgnored {
  f: ThreadsafeFunction<String, Promise<bool>>,
}

#[async_trait]
impl Ignored for SafetyIgnored {
  async fn ignore(&self, path: &str) -> bool {
    self
      .f
      .call_with_promise(path.to_string())
      .await
      .unwrap_or_default()
  }
}

type JsCallback = ThreadsafeFunction<FnArgs<(Option<napi::Error>, Vec<String>, Vec<String>)>, ()>;
type JsCallbackUndelayed = ThreadsafeFunction<String, ()>;

#[napi(object, object_to_js = false)]
pub struct NativeWatcherOptions {
  pub follow_symlinks: Option<bool>,

  pub poll_interval: Option<u32>,

  pub aggregate_timeout: Option<u32>,

  #[napi(ts_type = "(path: string) => Promise<boolean>")]
  /// A function that will be called with the path of a file or directory that is ignored.
  pub ignored: Option<ThreadsafeFunction<String, Promise<bool>>>,
}

#[napi]
pub struct NativeWatcher {
  watcher: FsWatcher,
  close: bool,
}

#[napi]
impl NativeWatcher {
  #[napi(constructor)]
  pub fn new(options: NativeWatcherOptions) -> Self {
    let ignored = options
      .ignored
      .map(|f| Box::new(SafetyIgnored { f: f.clone() }) as Box<dyn Ignored>);

    let watcher = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: options.follow_symlinks.unwrap_or(true),
        poll_interval: options.poll_interval,
        aggregate_timeout: options.aggregate_timeout,
      },
      ignored,
    );

    Self {
      watcher,
      close: false,
    }
  }

  #[napi]
  /// # Safety
  ///
  /// This function is unsafe because it uses `&mut self` to call the watcher asynchronously.
  /// It's important to ensure that the watcher is not used in any other places before this function is finished.
  /// You must ensure that the watcher not call watch, close or pause in the same time, otherwise it may lead to undefined behavior.
  pub async unsafe fn watch(
    &mut self,
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    #[napi(
      ts_arg_type = "(err: Error | null, changedFiles: string[], removedFiles: string[]) => void"
    )]
    callback: JsCallback,
    #[napi(ts_arg_type = "(path: string) => void")] callback_undelayed: JsCallbackUndelayed,
  ) -> napi::Result<()> {
    if self.close {
      return Err(napi::Error::from_reason(
        "Watcher has been closed, cannot watch again.",
      ));
    }
    let js_event_handler = JsEventHandler::new(callback, callback_undelayed);

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

    self
      .watcher
      .watch(
        file_updater,
        directories_updater,
        missing_updater,
        Box::new(js_event_handler),
      )
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

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
    self.close = true;
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
  callback: JsCallback,
  callback_undelayed: JsCallbackUndelayed,
}

impl JsEventHandler {
  fn new(callback: JsCallback, callback_undelayed: JsCallbackUndelayed) -> Self {
    Self {
      callback,
      callback_undelayed,
    }
  }
}

#[async_trait::async_trait]
impl rspack_fs::EventHandler for JsEventHandler {
  async fn on_event_handle(
    &self,
    changed_files: std::collections::HashSet<String>,
    deleted_files: std::collections::HashSet<String>,
  ) -> rspack_error::Result<()> {
    let changed_files_vec: Vec<String> = changed_files.into_iter().collect();
    let deleted_files_vec: Vec<String> = deleted_files.into_iter().collect();

    self
      .callback
      .call_with_sync(FnArgs {
        data: (None, changed_files_vec, deleted_files_vec),
      })
      .await
  }

  async fn on_change(&self, changed_file: String) -> rspack_error::Result<()> {
    self.callback_undelayed.call_with_sync(changed_file).await
  }
}
