use std::{future::Future, pin::Pin, time::Duration};

use rspack_tasks::{WorkerConsumer, WorkerDispatchError};

/// Drives a native MPMC consumer with an environment-local callback adapter.
///
/// The dispatcher owns queueing and job/result lifetime. The caller supplies only the conversion
/// and callback future for its N-API environment, so this loop can be reused by domains other than
/// JS loaders without teaching it about compiler or loader types.
pub async fn drive_worker_consumer<I, H>(
  consumer: WorkerConsumer<I, I>,
  mut handle: H,
) -> Result<(), WorkerDispatchError>
where
  I: Send + 'static,
  H: for<'a> FnMut(
    u64,
    Duration,
    &'a mut I,
  ) -> Pin<Box<dyn Future<Output = Result<(), WorkerDispatchError>> + Send + 'a>>,
{
  while let Some(mut job) = consumer.recv().await {
    if job.is_cancelled() {
      job.fail(WorkerDispatchError::Cancelled);
      continue;
    }
    let job_id = job.id();
    let queue_duration = job.queue_duration();
    match handle(job_id, queue_duration, job.input_mut()).await {
      Ok(()) => {
        let output = job.take_input();
        job.complete(output);
      }
      Err(error) => job.fail(error),
    }
  }
  Ok(())
}
