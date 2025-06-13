use std::boxed::Box;

use async_trait::async_trait;
use napi::bindgen_prelude::{FnArgs, Promise};
use napi_derive::*;
use rspack_fs::{FsWatcher, FsWatcherOptions, Ignored};
use rspack_napi::threadsafe_function::ThreadsafeFunction;

struct SaftyIgnored {
  f: ThreadsafeFunction<String, Promise<bool>>,
}

#[async_trait]
impl Ignored for SaftyIgnored {
  async fn ignore(&self, path: &str) -> bool {
    self
      .f
      .call_with_promise(path.to_string())
      .await
      .unwrap_or_default()
  }
}

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
}

#[napi]
impl NativeWatcher {
  #[napi(constructor)]
  pub fn new(options: NativeWatcherOptions) -> Self {
    let ignored = options
      .ignored
      .map(|f| Box::new(SaftyIgnored { f: f.clone() }) as Box<dyn Ignored>);

    let watcher = FsWatcher::new(
      FsWatcherOptions {
        follow_symlinks: options.follow_symlinks.unwrap_or(true),
        poll_interval: options.poll_interval,
        aggregate_timeout: options.aggregate_timeout,
      },
      ignored,
    );

    Self { watcher }
  }

  #[napi]
  pub async unsafe fn watch(
    &mut self,
    files: (Vec<String>, Vec<String>),
    directories: (Vec<String>, Vec<String>),
    missing: (Vec<String>, Vec<String>),
    #[napi(
      ts_arg_type = "(err: Error | null, changedFiles: string[], removedFiles: string[]) => void"
    )]
    callback: ThreadsafeFunction<FnArgs<(Option<napi::Error>, Vec<String>, Vec<String>)>, ()>,
    #[napi(ts_arg_type = "(path: string) => void")] callback_undelayed: ThreadsafeFunction<
      String,
      (),
    >,
    // callback: ThreadsafeFunction<String, bool>,
  ) -> napi::Result<()> {
    let js_event_handler = JsEventHandler::new(callback, callback_undelayed);
    self
      .watcher
      .watch(files, directories, missing, Box::new(js_event_handler))
      .await
      .map_err(|e| napi::Error::from_reason(e.to_string()))?;

    Ok(())
  }

  #[napi]
  pub fn close(&mut self) -> napi::Result<()> {
    // Implement the close method to stop the watcher
    // This is a placeholder, actual implementation may vary
    // TODO: Implement the close method to stop the watcher
    self.watcher.close().unwrap();
    Ok(())
  }
}

struct JsEventHandler {
  callback: ThreadsafeFunction<FnArgs<(Option<napi::Error>, Vec<String>, Vec<String>)>, ()>,
  _callback_undelayed: ThreadsafeFunction<String, ()>,
}

impl JsEventHandler {
  fn new(
    callback: ThreadsafeFunction<FnArgs<(Option<napi::Error>, Vec<String>, Vec<String>)>, ()>,
    callback_undelayed: ThreadsafeFunction<String, ()>,
  ) -> Self {
    Self {
      callback,
      _callback_undelayed: callback_undelayed,
    }
  }
}

#[async_trait::async_trait]
impl rspack_fs::EventHandler for JsEventHandler {
  async fn on_event_handle(
    &self,
    changed_files: std::collections::HashSet<String>,
    deleted_files: std::collections::HashSet<String>,
  ) {
    let changed_files_vec: Vec<String> = changed_files.into_iter().collect();
    let deleted_files_vec: Vec<String> = deleted_files.into_iter().collect();

    self
      .callback
      .call_with_sync(FnArgs {
        data: (None, changed_files_vec.clone(), deleted_files_vec.clone()),
      })
      .await
      .unwrap();
  }
}
