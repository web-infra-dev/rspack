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

/// Process-wide native worker payload. The generic queue has one payload kind today; when another
/// native worker task is added this can become an enum without changing the queue lifecycle.
pub(crate) struct WorkerTaskPayload {
  pub(crate) loader_context: LoaderContext<RunnerContext>,
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

/// Owns one Rust loader context received from the process-wide native MPMC queue.
#[napi]
pub struct WorkerTask {
  job: Option<Box<NativeWorkerJob>>,
  context_taken: bool,
}

impl WorkerTask {
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
    let job = self
      .job
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    let loader_compilation = job.input().loader_context.context.compilation;
    let mut context: JsLoaderContext = (&mut job.input_mut().loader_context)
      .try_into()
      .map_err(|error: rspack_error::Error| napi::Error::from_reason(error.to_string()))?;
    context.module.set_loader_compilation(loader_compilation);
    context.hook_extensions = job.input_mut().hook_extensions.take();
    self.context_taken = true;
    Ok(context)
  }

  #[napi]
  pub fn get_compilation(&self) -> napi::Result<JsCompilationWrapper> {
    let job = self
      .job
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    Ok(JsCompilationWrapper::new(
      job.input().loader_context.context.compilation.as_ref(),
    ))
  }

  #[napi]
  pub fn get_resolver(
    &self,
    options: Option<RawResolveOptionsWithDependencyType>,
  ) -> napi::Result<JsResolver> {
    let job = self
      .job
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
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
      job
        .input()
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
    let job = self
      .job
      .as_ref()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    let logger = job
      .input()
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
    let mut job = self.take_job()?;
    ModuleObject::cleanup_by_compiler_id(&job.input().loader_context.context.compiler_id);
    let loaders_without_pitch = context
      .loader_items
      .iter()
      .zip(&job.input().loader_context.loader_items)
      .filter_map(|(js_item, item)| js_item.no_pitch.then(|| item.path().to_string()))
      .collect();
    job.input_mut().loaders_without_pitch = loaders_without_pitch;
    if let Err(error) = merge_loader_context(&mut job.input_mut().loader_context, context) {
      job.fail(error);
      return Ok(());
    }
    job.complete();
    Ok(())
  }

  #[napi]
  pub fn fail(&mut self, error: String) -> napi::Result<()> {
    let job = self.take_job()?;
    ModuleObject::cleanup_by_compiler_id(&job.input().loader_context.context.compiler_id);
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
