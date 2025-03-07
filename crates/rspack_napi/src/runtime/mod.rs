mod executor;
mod runtime;
mod waker;

use std::{future::Future, ptr};

use napi::{sys, Env};
use waker::{LocalWaker, WakerEvent};

use self::runtime::LocalRuntime;
use crate::next_tick::QUEUE;

extern "C" fn on_wake(
  _env: sys::napi_env,
  _callback_info: sys::napi_callback_info,
) -> sys::napi_value {
  while let Some(f) = QUEUE.pop() {
    f();
  }
  ptr::null_mut()
}

/// Schedule a future to run asynchronously on the local JavaScript thread.
/// The future's execution will not block the local thread.
pub fn spawn_async_local(env: &Env, future: impl Future + 'static) -> napi::Result<()> {
  // Add a future to the future pool to be executed
  // whenever the Nodejs event loop is free to do so
  LocalRuntime::queue_future(future);

  // If there are tasks in flight then the executor
  // is already running and should be reused
  if LocalRuntime::futures_count() > 1 {
    return Ok(());
  }

  // The futures executor runs on the main thread thread but
  // the waker runs on another thread.
  //
  // The main thread executor will run the contained futures
  // and as soon as they stall (e.g. waiting for a channel, timer, etc),
  // the executor will immediately yield back to the JavaScript event loop.
  //
  // This "parks" the executer, which normally means the thread
  // is block - however we cannot do that here so instead, there
  // is a sacrificial "waker" thread who's only job is to sleep/wake and
  // signal to Nodejs that futures need to be run.
  //
  // The waker thread notifies the main thread of pending work by
  // running the futures executor within a threadsafe function
  let jsfn = env.create_function::<(), ()>("onwake", on_wake)?;

  let tsfn = jsfn
    .build_threadsafe_function()
    .max_queue_size::<0>()
    .callee_handled::<false>()
    .weak::<true>()
    .build_callback(|ctx| {
      let thread_notify = ctx.value;
      let done = LocalRuntime::run_until_stalled(thread_notify);

      if done {
        LocalWaker::send(WakerEvent::Done);
      } else {
        LocalWaker::send(WakerEvent::Next);
      }

      Ok(())
    })?;

  println!("send");
  LocalWaker::send(WakerEvent::Init(tsfn));

  Ok(())
}
