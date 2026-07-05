use std::{future::pending, sync::Arc};

use rspack_tasks::{
  CURRENT_COMPILER_CONTEXT, CompilerContext, block_on, fetch_new_dependency_id, spawn,
  spawn_blocking, spawn_in_compiler_context, within_compiler_context,
};

#[test]
fn block_on_runs_spawned_task() {
  let value = block_on(async { spawn(async { 41 }).await.unwrap() + 1 });

  assert_eq!(value, 42);
}

#[test]
fn spawn_blocking_resolves_on_runtime() {
  let value = block_on(async { spawn_blocking(|| 42).await.unwrap() });

  assert_eq!(value, 42);
}

#[test]
fn spawn_blocking_runs_many_tasks() {
  let value = block_on(async {
    let handles = (0..64)
      .map(|index| spawn_blocking(move || index))
      .collect::<Vec<_>>();

    let mut sum = 0;
    for handle in handles {
      sum += handle.await.unwrap();
    }
    sum
  });

  assert_eq!(value, (0..64).sum());
}

#[test]
fn join_handle_reports_panics() {
  let error = block_on(async { spawn(async { panic!("boom") }).await.unwrap_err() });

  assert!(error.is_panic());
  let payload = error.into_panic();
  assert_eq!(payload.downcast_ref::<&str>(), Some(&"boom"));
}

#[test]
fn abort_cancels_pending_task() {
  let handle = spawn(pending::<()>());
  handle.abort();

  let error = block_on(async { handle.await.unwrap_err() });

  assert!(error.is_cancelled());
}

#[test]
fn spawned_tasks_keep_compiler_context() {
  let compiler_context = Arc::new(CompilerContext::new());
  compiler_context.set_dependency_id(7);

  let value = block_on(within_compiler_context(compiler_context, async {
    spawn_in_compiler_context(async { fetch_new_dependency_id() })
      .await
      .unwrap()
  }));

  assert_eq!(value, 7);
}

#[test]
fn try_with_reads_current_compiler_context() {
  assert!(CURRENT_COMPILER_CONTEXT.try_with(|_| ()).is_err());

  let compiler_context = Arc::new(CompilerContext::new());
  compiler_context.set_dependency_id(7);

  let value = block_on(within_compiler_context(compiler_context, async {
    CURRENT_COMPILER_CONTEXT
      .try_with(|compiler_context| compiler_context.dependency_id())
      .unwrap()
  }));

  assert_eq!(value, 7);
}
