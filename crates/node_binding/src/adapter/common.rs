use std::sync::Arc;

use dashmap::DashMap;
use napi::threadsafe_function::{ErrorStrategy, ThreadsafeFunction};
use once_cell::sync::Lazy;
use tokio::sync::oneshot::Sender;

use super::{OnLoadResult, OnResolveResult};

pub type ThreadsafeRspackCallback = ThreadsafeFunction<String, ErrorStrategy::CalleeHandled>;

pub static REGISTERED_BUILD_START_SENDERS: Lazy<Arc<DashMap<usize, Sender<()>>>> =
  Lazy::new(Default::default);

pub static REGISTERED_LOAD_SENDERS: Lazy<Arc<DashMap<usize, Sender<Option<OnLoadResult>>>>> =
  Lazy::new(Default::default);

pub static REGISTERED_RESOLVE_SENDERS: Lazy<Arc<DashMap<usize, Sender<Option<OnResolveResult>>>>> =
  Lazy::new(Default::default);

pub static REGISTERED_BUILD_END_SENDERS: Lazy<Arc<DashMap<usize, Sender<()>>>> =
  Lazy::new(Default::default);
