use std::sync::LazyLock;

use async_channel::{Receiver, Sender};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use rspack_core::{
  DependencyCategory, LoaderContext, LogType, Logger, ResolveOptionsWithDependencyType,
  RunnerContext,
};
use rspack_tasks::{WorkerDispatchError, WorkerJob, WorkerJobFailure};

use crate::{
  compilation::JsCompilationWrapper,
  module::ModuleObject,
  options::raw_resolve::{
    RawResolveOptionsWithDependencyType, normalize_raw_resolve_options_with_dependency_type,
  },
  plugins::js_loader::{JsLoaderContext, merge_loader_context},
  resolver::JsResolver,
};

pub(crate) enum WorkerTaskPayload {
  Loader(LoaderTaskPayload),
  Function(FunctionTaskPayload),
}

#[derive(Clone)]
#[napi(object)]
pub struct JsWorkerFunction {
  pub version: u32,
  pub compiler_id: u32,
  pub hook: String,
  /// Versioned, resolved descriptor graph encoded by the shared JS value codec.
  pub value: String,
}

pub(crate) struct FunctionTaskPayload {
  pub(crate) functions: Vec<JsWorkerFunction>,
  pub(crate) data: Option<Box<crate::normal_module_factory::JsResolveData>>,
  pub(crate) result: Option<bool>,
}

#[napi(object, object_from_js = false)]
pub struct JsFunctionTask {
  pub functions: Vec<JsWorkerFunction>,
  pub data: crate::normal_module_factory::JsResolveData,
}

pub(crate) struct LoaderTaskPayload {
  pub(crate) loader_context: Box<LoaderContext<RunnerContext>>,
  pub(crate) loaders_without_pitch: Vec<String>,
  pub(crate) hook_extensions: Option<String>,
}

type NativeWorkerJob = WorkerJob<WorkerTaskPayload, rspack_error::Error>;
type NativeWorkerSender = Sender<Box<NativeWorkerJob>>;
type NativeWorkerReceiver = Receiver<Box<NativeWorkerJob>>;
type NativeWorkerJobFailure = WorkerJobFailure<WorkerTaskPayload, rspack_error::Error>;

static WORKER_QUEUE: LazyLock<(NativeWorkerSender, NativeWorkerReceiver)> =
  LazyLock::new(async_channel::unbounded);

pub(crate) async fn dispatch_worker_task(
  input: Box<WorkerTaskPayload>,
) -> std::result::Result<Box<WorkerTaskPayload>, NativeWorkerJobFailure> {
  NativeWorkerJob::dispatch(&WORKER_QUEUE.0, input).await
}

/// Owns one Rust loader or function task received from the process-wide native MPMC queue.
#[napi]
pub struct WorkerTask {
  job: Option<Box<NativeWorkerJob>>,
  context_taken: bool,
}

impl WorkerTask {
  fn loader(&self) -> napi::Result<&LoaderTaskPayload> {
    match self
      .job
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?
      .input()
    {
      WorkerTaskPayload::Loader(payload) => Ok(payload),
      _ => Err(napi::Error::from_reason("Expected a loader worker task")),
    }
  }

  fn loader_mut(&mut self) -> napi::Result<&mut LoaderTaskPayload> {
    match self
      .job
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?
      .input_mut()
    {
      WorkerTaskPayload::Loader(payload) => Ok(payload),
      _ => Err(napi::Error::from_reason("Expected a loader worker task")),
    }
  }
  fn take_job(&mut self) -> napi::Result<Box<NativeWorkerJob>> {
    self
      .job
      .take()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))
  }
}

#[napi]
impl WorkerTask {
  /// Materializes the ordinary JsLoaderContext DTO in the worker isolate. Until this boundary the
  /// queue contains only the canonical Rust LoaderContext.
  #[napi]
  pub fn take_context(&mut self) -> napi::Result<JsLoaderContext> {
    if self.context_taken {
      return Err(napi::Error::from_reason(
        "Worker task context has already been taken",
      ));
    }
    let payload = self.loader_mut()?;
    let loader_compilation = payload.loader_context.context.compilation;
    let mut context: JsLoaderContext = payload
      .loader_context
      .as_mut()
      .try_into()
      .map_err(|error: rspack_error::Error| napi::Error::from_reason(error.to_string()))?;
    context.module.set_loader_compilation(loader_compilation);
    context.hook_extensions = payload.hook_extensions.take();
    self.context_taken = true;
    Ok(context)
  }

  #[napi(ts_return_type = "JsCompilation")]
  pub fn get_compilation(&self) -> napi::Result<JsCompilationWrapper> {
    let payload = self.loader()?;
    Ok(JsCompilationWrapper::new(
      payload.loader_context.context.compilation.as_ref(),
    ))
  }

  #[napi]
  pub fn get_resolver(
    &self,
    options: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolver> {
    let payload = self.loader()?;
    let options = match options {
      Some(options) => normalize_raw_resolve_options_with_dependency_type(Some(options), false)
        .map_err(|error| napi::Error::from_reason(error.to_string()))?,
      None => ResolveOptionsWithDependencyType {
        resolve_options: None,
        resolve_to_context: false,
        dependency_category: DependencyCategory::Unknown,
      },
    };
    Ok(JsResolver::new(
      payload
        .loader_context
        .context
        .compilation
        .as_ref()
        .resolver_factory
        .get(options),
    ))
  }

  #[napi]
  pub fn log(&self, name: String, log_type: String, message: Option<String>) -> napi::Result<()> {
    let payload = self.loader()?;
    let logger = payload
      .loader_context
      .context
      .compilation
      .as_ref()
      .get_logger(name);
    let message = message.unwrap_or_default();
    logger.raw(match log_type.as_str() {
      "error" => LogType::Error {
        message,
        trace: Vec::new(),
      },
      "warn" => LogType::Warn {
        message,
        trace: Vec::new(),
      },
      "info" => LogType::Info { message },
      "debug" => LogType::Debug { message },
      "trace" => LogType::Trace {
        message,
        trace: Vec::new(),
      },
      "group" => LogType::Group { message },
      "groupCollapsed" => LogType::GroupCollapsed { message },
      "groupEnd" => LogType::GroupEnd,
      "clear" => LogType::Clear,
      "status" => LogType::Status { message },
      _ => LogType::Log { message },
    });
    Ok(())
  }

  #[napi]
  pub fn complete(&mut self, context: JsLoaderContext) -> napi::Result<()> {
    self.loader()?;
    if !self.context_taken {
      return Err(napi::Error::from_reason(
        "Worker task context has not been taken",
      ));
    }
    let mut job = self.take_job()?;
    let WorkerTaskPayload::Loader(payload) = job.input_mut() else {
      unreachable!()
    };
    ModuleObject::cleanup_by_compiler_id(&payload.loader_context.context.compiler_id);
    payload.loaders_without_pitch = context
      .loader_items
      .iter()
      .zip(&payload.loader_context.loader_items)
      .filter(|(js_item, _)| js_item.no_pitch)
      .map(|(_, item)| item.path().to_string())
      .collect();
    if let Err(error) = merge_loader_context(&mut payload.loader_context, context) {
      job.fail(error);
      return Ok(());
    }
    job.complete();
    Ok(())
  }

  #[napi(getter)]
  pub fn kind(&self) -> napi::Result<&'static str> {
    let job = self
      .job
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    Ok(match job.input() {
      WorkerTaskPayload::Loader(_) => "loader",
      WorkerTaskPayload::Function(_) => "function",
    })
  }

  #[napi]
  pub fn take_function(&mut self) -> napi::Result<JsFunctionTask> {
    let job = self
      .job
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    let WorkerTaskPayload::Function(payload) = job.input_mut() else {
      return Err(napi::Error::from_reason("Expected a function worker task"));
    };
    let data = payload.data.take().ok_or_else(|| {
      napi::Error::from_reason("Worker function arguments have already been taken")
    })?;
    self.context_taken = true;
    Ok(JsFunctionTask {
      functions: payload.functions.clone(),
      data: *data,
    })
  }

  #[napi]
  pub fn complete_function(
    &mut self,
    data: crate::normal_module_factory::JsResolveData,
    result: Option<bool>,
  ) -> napi::Result<()> {
    let job = self
      .job
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    let WorkerTaskPayload::Function(payload) = job.input_mut() else {
      return Err(napi::Error::from_reason("Expected a function worker task"));
    };
    if !self.context_taken {
      return Err(napi::Error::from_reason(
        "Worker function arguments have not been taken",
      ));
    }
    payload.data = Some(Box::new(data));
    payload.result = result;
    self.take_job()?.complete();
    Ok(())
  }

  #[napi]
  pub fn fail(&mut self, error: String) -> napi::Result<()> {
    let job = self.take_job()?;
    if let WorkerTaskPayload::Loader(payload) = job.input() {
      ModuleObject::cleanup_by_compiler_id(&payload.loader_context.context.compiler_id);
    }
    job.fail(rspack_error::error!(error));
    Ok(())
  }
}

/// Every JavaScript worker loops around this receive operation, so all workers compete for the
/// same unbounded MPMC receiver without registration or per-pool state.
#[napi]
pub fn recv_worker_task(env: &Env) -> napi::Result<PromiseRaw<'_, WorkerTask>> {
  rspack_napi::runtime::promise_from_future(env, async move {
    loop {
      let job = WORKER_QUEUE
        .1
        .recv()
        .await
        .map_err(|_| napi::Error::from_reason(WorkerDispatchError::Closed.to_string()))?;
      if job.is_cancelled() {
        job.fail_dispatch(WorkerDispatchError::Cancelled);
        continue;
      }
      return Ok(WorkerTask {
        job: Some(job),
        context_taken: false,
      });
    }
  })
}
