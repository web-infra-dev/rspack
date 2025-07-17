use std::{
  sync::mpsc::{channel, Sender},
  thread,
};

use napi::{
  threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
  Status,
};
use once_cell::unsync::Lazy;

use super::executor::{wait_for_wake, ThreadNotify, ThreadNotifyRef};

thread_local! {
  static WAKER_THREAD: Lazy<Sender<WakerEvent>> = Lazy::new(LocalWaker::start_waker_thread);
}

pub type WakerInit = ThreadsafeFunction<ThreadNotifyRef, (), (), Status, false, true, 0>;

pub enum WakerEvent {
  Init(WakerInit),
  Next,
  Done,
}

/// The futures waker that coordinates with the futures executor to notify
/// the main thread to resume execution of futures.
///
/// The waker is implemented as a dedicated system thread which is parked
/// by the local futures executor. Futures (like channel, timers) will
/// call the wake() method Futures Waker trait.
///
/// This gives it some level of portability - for instance any utilities
/// from the "async_std" crate will work however most things from Tokio
/// won't work.
///
/// Once woken up, the waker resumes execution of futures on the JavaScript
/// thread by triggering a napi threadsafe function which executes a callback
/// that runs on the main JavaScript thread. This callback is used to poll
/// the futures in the local pool.
pub struct LocalWaker;

impl LocalWaker {
  pub fn send(event: WakerEvent) {
    WAKER_THREAD
      .with(|tx| tx.send(event))
      .expect("Unable to communicate with waker");
  }

  fn start_waker_thread() -> Sender<WakerEvent> {
    let (tx, rx) = channel();

    thread::spawn(move || {
      let thread_notify = ThreadNotify::new();
      let mut handle = None::<WakerInit>;

      while let Ok(event) = rx.recv() {
        match event {
          WakerEvent::Init(incoming) => {
            if handle.replace(incoming).is_some() {
              panic!("Handle already init");
            };
            let Some(ref handle) = handle else {
              panic!("No handle");
            };
            handle.call(thread_notify.clone(), ThreadsafeFunctionCallMode::Blocking);
          }
          WakerEvent::Next => {
            wait_for_wake(&thread_notify);
            let Some(ref handle) = handle else {
              panic!("No handle");
            };
            handle.call(thread_notify.clone(), ThreadsafeFunctionCallMode::Blocking);
          }
          WakerEvent::Done => {
            if let Some(handle) = handle.take() {
              drop(handle);
            }
          }
        };
      }
    });

    tx
  }
}
