// borrow the ideas from turbo_tasks https://github.com/vercel/next.js/blob/678ef8b5650871a730ca14c480c762ca53716575/turbopack/crates/turbo-tasks/src/manager.rs#L1
// which creates a implicit compiler context to support isolated parallel compiler state
use std::{
  future::Future,
  sync::{
    Arc,
    atomic::{AtomicU32, AtomicUsize},
  },
};

use tokio::{
  task::{JoinHandle, futures::TaskLocalFuture},
  task_local,
};

// don't overuse this and put everything here, it's mostly used for store isolated id generator
#[derive(Debug)]
pub struct CompilerContext {
  dependenc_id_generator: AtomicU32,
  exports_info_artifact_ptr: AtomicUsize,
}

task_local! {
  // implicit COMPIlER_CONTEXT for current running compiler, every compiler has its own isolated compiler context
 pub static CURRENT_COMPILER_CONTEXT: Arc<CompilerContext>;
}
#[allow(clippy::new_without_default)]
impl CompilerContext {
  pub fn new() -> Self {
    Self {
      dependenc_id_generator: AtomicU32::new(0),
      exports_info_artifact_ptr: AtomicUsize::new(0),
    }
  }
  pub fn fetch_new_dependency_id(&self) -> u32 {
    self
      .dependenc_id_generator
      .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
  }
  pub fn dependency_id(&self) -> u32 {
    self
      .dependenc_id_generator
      .load(std::sync::atomic::Ordering::SeqCst)
  }
  pub fn set_dependency_id(&self, id: u32) {
    self
      .dependenc_id_generator
      .store(id, std::sync::atomic::Ordering::SeqCst);
  }

  pub fn exports_info_artifact_ptr(&self) -> Option<usize> {
    let ptr = self
      .exports_info_artifact_ptr
      .load(std::sync::atomic::Ordering::SeqCst);
    (ptr != 0).then_some(ptr)
  }

  pub fn set_exports_info_artifact_ptr(&self, ptr: Option<usize>) {
    self
      .exports_info_artifact_ptr
      .store(ptr.unwrap_or_default(), std::sync::atomic::Ordering::SeqCst);
  }
}

pub fn fetch_new_dependency_id() -> u32 {
  CURRENT_COMPILER_CONTEXT.get().fetch_new_dependency_id()
}
pub fn get_current_dependency_id() -> u32 {
  CURRENT_COMPILER_CONTEXT.get().dependency_id()
}
pub fn set_current_dependency_id(id: u32) {
  CURRENT_COMPILER_CONTEXT.get().set_dependency_id(id);
}

pub fn with_current_exports_info_artifact<T>(f: impl FnOnce(Option<usize>) -> T) -> T {
  let ptr = CURRENT_COMPILER_CONTEXT
    .try_with(|ctx| ctx.exports_info_artifact_ptr())
    .ok()
    .flatten();
  f(ptr)
}

pub fn within_compiler_context<F>(
  compiler_context: Arc<CompilerContext>,
  f: F,
) -> TaskLocalFuture<Arc<CompilerContext>, F>
where
  F: Future,
{
  CURRENT_COMPILER_CONTEXT.scope(compiler_context, f)
}
pub fn within_compiler_context_sync<F, R>(compiler_context: Arc<CompilerContext>, f: F) -> R
where
  F: FnOnce() -> R,
{
  CURRENT_COMPILER_CONTEXT.sync_scope(compiler_context, f)
}

// this is only used for testing rust builder api, we need to find better api in the future
/// For test use only.
pub fn within_compiler_context_for_testing_sync<F, R>(f: F) -> R
where
  F: FnOnce() -> R,
{
  CURRENT_COMPILER_CONTEXT.sync_scope(Arc::new(CompilerContext::new()), f)
}
/// For test use only.
pub fn within_compiler_context_for_testing<F>(f: F) -> TaskLocalFuture<Arc<CompilerContext>, F>
where
  F: Future,
{
  CURRENT_COMPILER_CONTEXT.scope(Arc::new(CompilerContext::new()), f)
}

pub fn spawn_in_compiler_context<F>(future: F) -> JoinHandle<F::Output>
where
  F: Future + Send + 'static,
  F::Output: Send + 'static,
{
  let compiler_context = CURRENT_COMPILER_CONTEXT.get();

  tokio::spawn(CURRENT_COMPILER_CONTEXT.scope(compiler_context, future))
}
