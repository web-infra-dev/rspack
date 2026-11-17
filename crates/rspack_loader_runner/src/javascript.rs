use std::sync::Mutex;

use rspack_error::{Result, error};
use rspack_js_tasks::{JsTask, LoaderTask, LoaderTaskTiming, js_task_dispatcher};
use rustc_hash::{FxHashMap, FxHashSet};
use tokio::sync::oneshot;

use crate::{LoaderContext, LoaderExecutionKind, State};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum ExecutionMode {
  #[default]
  Main,
  Parallel,
  Strict,
}

/// Per-compiler JavaScript loader metadata, supplied by the binding. The runner
/// owns chain execution and pitch caching without retaining a JS environment.
#[derive(Debug)]
pub struct JsLoaderRuntime {
  main_object_handle: u32,
  parallel_loaders: Mutex<FxHashMap<String, ExecutionMode>>,
  loaders_without_pitch: Mutex<FxHashSet<String>>,
}

impl JsLoaderRuntime {
  pub fn new(main_object_handle: u32) -> Self {
    Self {
      main_object_handle,
      parallel_loaders: Default::default(),
      loaders_without_pitch: Default::default(),
    }
  }

  pub fn register_loader(&self, ident: String, parallel: bool, strict: bool) {
    self
      .parallel_loaders
      .lock()
      .expect("loader references lock poisoned")
      .insert(
        ident,
        if !parallel {
          ExecutionMode::Main
        } else if strict {
          ExecutionMode::Strict
        } else {
          ExecutionMode::Parallel
        },
      );
  }

  fn execution_mode(&self, query: Option<&str>) -> ExecutionMode {
    let Some(ident) = query.and_then(|query| query.strip_prefix("??")) else {
      return ExecutionMode::Main;
    };
    self
      .parallel_loaders
      .lock()
      .expect("loader references lock poisoned")
      .get(ident)
      .copied()
      .unwrap_or_default()
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
      let current = self.loader_index as usize;
      let mode_at = |index: usize| runtime.execution_mode(self.loader_items[index].query());
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
    let (tx, rx) = oneshot::channel();
    let timing = LoaderTaskTiming::new(
      self.resource(),
      self.current_loader().path().as_str(),
      if self.state() == State::Pitching {
        "pitch"
      } else {
        "normal"
      },
      mode != ExecutionMode::Main,
    );
    let mut task = LoaderTask::new(
      self.take_owned(),
      runtime.main_object_handle,
      start as u32,
      end as u32,
      tx,
    );
    task.strict = mode == ExecutionMode::Strict;
    task.timing = timing.clone();
    let task = JsTask::Loader(task);
    if mode != ExecutionMode::Main {
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
