use std::sync::Mutex;

use rspack_error::{Result, error};
use rspack_js_tasks::{JsTask, LoaderTask, LoaderTaskMode, LoaderTaskTiming, js_task_dispatcher};
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::sync::oneshot;

use crate::{LoaderContext, LoaderExecutionKind, State};

/// Per-compiler JavaScript loader metadata, supplied by the binding. The runner
/// owns chain execution and pitch caching without retaining a JS environment.
#[derive(Debug)]
pub struct JsLoaderRuntime {
  main_object_handle: u32,
  loader_modes: Mutex<FxHashMap<String, LoaderTaskMode>>,
  loaders_without_pitch: Mutex<FxHashSet<String>>,
}

impl JsLoaderRuntime {
  pub fn new(main_object_handle: u32) -> Self {
    Self {
      main_object_handle,
      loader_modes: Default::default(),
      loaders_without_pitch: Default::default(),
    }
  }

  pub fn register_loader(&self, ident: String, mode: LoaderTaskMode) {
    self
      .loader_modes
      .lock()
      .expect("loader references lock poisoned")
      .insert(ident, mode);
  }
}

impl<Context: Send + 'static> LoaderContext<Context> {
  pub(crate) async fn loader_yield(&mut self) -> Result<bool> {
    if self.current_loader().execution_kind() != LoaderExecutionKind::JavaScript {
      return Ok(false);
    }
    let runtime = self
      .current_loader()
      .loader()
      .js_runtime(&self.context)
      .ok_or_else(|| error!("JavaScript loader runtime is unavailable"))?;
    if self.state() == State::Pitching {
      let without_pitch = runtime
        .loaders_without_pitch
        .lock()
        .expect("loader pitch cache lock poisoned");
      let end = self
        .current_chain()
        .expect("pitching requires a current execution chain")
        .end();
      let start = self.loader_index as usize;
      let needs_pitch = self.loader_items[start..end]
        .iter()
        .any(|loader| !loader.pitch_executed() && !without_pitch.contains(loader.path().as_str()));
      if !needs_pitch {
        for loader in &self.loader_items[start..end] {
          loader.set_pitch_executed();
        }
        self.loader_index = end as i32;
        return Ok(true);
      }
    }
    let (start, end, mode) = {
      let modes = runtime
        .loader_modes
        .lock()
        .expect("loader references lock poisoned");
      let current = self.loader_index as usize;
      let mode_at = |index: usize| {
        self.loader_items[index]
          .query()
          .and_then(|query| query.strip_prefix("??"))
          .and_then(|ident| modes.get(ident).copied())
          .unwrap_or(LoaderTaskMode::Main)
      };
      let mode = mode_at(current);
      let chain = self.current_chain().expect("yield requires a loader chain");
      let mut start = current;
      let mut end = current + 1;
      while start > chain.start() && mode_at(start - 1) == mode {
        start -= 1;
      }
      while end < chain.end() && mode_at(end) == mode {
        end += 1;
      }
      (start, end, mode)
    };
    let parallel = mode != LoaderTaskMode::Main;
    let (tx, rx) = oneshot::channel();
    let timing = LoaderTaskTiming::new(
      self.resource(),
      self.current_loader().path().as_str(),
      if self.state() == State::Pitching {
        "pitch"
      } else {
        "normal"
      },
      parallel,
    );
    let mut task = LoaderTask::new(
      self.take_owned(),
      runtime.main_object_handle,
      mode,
      start as u32,
      end as u32,
      tx,
    );
    task.timing = timing.clone();
    let task = JsTask::Loader(task);
    if parallel {
      js_task_dispatcher().dispatch_worker(task);
    } else {
      js_task_dispatcher().dispatch_main(task).await;
    }
    let (context, pitches, result) = rx.await.expect("loader task must restore native state");
    let context = context
      .downcast::<Self>()
      .unwrap_or_else(|_| panic!("loader task must return its original context type"));
    self.restore_owned(*context);
    if !pitches.is_empty() {
      runtime
        .loaders_without_pitch
        .lock()
        .expect("loader pitch cache lock poisoned")
        .extend(pitches);
    }
    if let Some(timing) = timing {
      timing.finish(result.is_ok());
    }
    result?;
    Ok(true)
  }
}
