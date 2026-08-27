use std::sync::LazyLock;

use napi::bindgen_prelude::*;
use rspack_tasks::{WorkerDispatchError, WorkerDispatcher, WorkerJob};

// The envelope is intentionally task-kind agnostic while JS loaders are the only consumer.
// Add a task kind enum here and dispatch on it in the JS worker when a second task kind is added.
struct ScheduledWorkerTask {
  payload: Vec<u8>,
  error: Option<String>,
}

type ScheduledWorkerJob = WorkerJob<ScheduledWorkerTask, ScheduledWorkerTask>;
type WorkerTaskDispatcher = WorkerDispatcher<ScheduledWorkerTask, ScheduledWorkerTask>;

static WORKER_DISPATCHER: LazyLock<WorkerTaskDispatcher> =
  LazyLock::new(WorkerDispatcher::unbounded);

#[napi]
pub struct WorkerTask {
  job: Option<Box<ScheduledWorkerJob>>,
}

#[napi]
impl WorkerTask {
  #[napi]
  pub fn take_payload(&mut self) -> napi::Result<Buffer> {
    let job = self
      .job
      .as_mut()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    Ok(Buffer::from(std::mem::take(&mut job.input_mut().payload)))
  }

  #[napi]
  pub fn complete(&mut self, payload: Buffer) -> napi::Result<()> {
    let mut job = self
      .job
      .take()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    job.input_mut().payload = payload.into();
    let output = job.take_input();
    job.complete(output);
    Ok(())
  }

  #[napi]
  pub fn fail(&mut self, error: String) -> napi::Result<()> {
    let mut job = self
      .job
      .take()
      .ok_or_else(|| napi::Error::from_reason("Worker task has already finished"))?;
    job.input_mut().error = Some(error);
    let output = job.take_input();
    job.complete(output);
    Ok(())
  }
}

#[napi]
pub fn recv_worker_task(env: &Env) -> napi::Result<PromiseRaw<'_, WorkerTask>> {
  rspack_napi::runtime::promise_from_future(env, async move {
    loop {
      let job = WORKER_DISPATCHER
        .recv()
        .await
        .ok_or_else(|| dispatch_error_to_napi(WorkerDispatchError::Closed))?;
      if job.is_cancelled() {
        job.fail(WorkerDispatchError::Cancelled);
        continue;
      }
      return Ok(WorkerTask { job: Some(job) });
    }
  })
}

#[napi]
pub fn dispatch_worker_task<'env>(
  env: &'env Env,
  payload: Buffer,
) -> napi::Result<PromiseRaw<'env, Buffer>> {
  rspack_napi::runtime::promise_from_future(env, async move {
    let task = WORKER_DISPATCHER
      .dispatch(Box::new(ScheduledWorkerTask {
        payload: payload.into(),
        error: None,
      }))
      .await
      .map_err(|failure| dispatch_error_to_napi(failure.error()))?;
    match task.error {
      Some(error) => Err(napi::Error::from_reason(error)),
      None => Ok(Buffer::from(task.payload)),
    }
  })
}

fn dispatch_error_to_napi(error: WorkerDispatchError) -> napi::Error {
  napi::Error::from_reason(error.to_string())
}
