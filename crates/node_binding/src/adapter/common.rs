use std::sync::Arc;

use dashmap::DashMap;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use once_cell::sync::Lazy;
use tokio::sync::oneshot::Sender;

pub type ThreadsafeRspackCallback = ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>;

pub static REGISTERED_BUILD_END_SENDERS: Lazy<Arc<DashMap<usize, Sender<()>>>> =
  Lazy::new(Default::default);
