//! Executor selection for native Rust compiler tasks.
//!
//! The opt-in `goexec` feature requires a goexec runtime around compilation.
//! Tokio task-local scopes and synchronization are shared by both backends.
#[cfg(feature = "goexec")]
pub use goexec::{JoinError, JoinHandle, Runtime, blocking, spawn};
#[cfg(not(feature = "goexec"))]
pub use tokio::{
  runtime::Runtime,
  spawn,
  task::{JoinError, JoinHandle, spawn_blocking},
};

#[cfg(not(feature = "goexec"))]
#[inline]
pub fn blocking<F: FnOnce() -> R, R>(f: F) -> R {
  f()
}

#[cfg(feature = "goexec")]
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
  F: FnOnce() -> R + Send + 'static,
  R: Send + 'static,
{
  // These callers offload CPU-heavy destruction, not syscalls. Keep it within
  // the executor's CPU permit budget; do not mark it as a blocking syscall.
  spawn(async move { f() })
}
