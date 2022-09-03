use dashmap::DashMap;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use once_cell::sync::Lazy;
use tokio::sync::oneshot::Sender;

pub type ThreadsafeRspackCallback<T = String> = ThreadsafeFunction<T, ErrorStrategy::CalleeHandled>;

pub static REGISTERED_PROCESS_ASSETS_SENDERS: Lazy<DashMap<usize, Sender<()>>> =
  Lazy::new(Default::default);

pub static REGISTERED_DONE_SENDERS: Lazy<DashMap<usize, Sender<()>>> = Lazy::new(Default::default);
