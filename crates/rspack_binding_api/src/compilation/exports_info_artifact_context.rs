use std::{
  collections::HashMap,
  future::Future,
  sync::{LazyLock, Mutex},
};

use rspack_core::{CompilerId, ExportsInfoArtifact};

static CURRENT_EXPORTS_INFO_ARTIFACT_STACK: LazyLock<Mutex<HashMap<CompilerId, Vec<usize>>>> =
  LazyLock::new(|| Mutex::new(HashMap::default()));

struct ExportsInfoArtifactScopeGuard {
  compiler_id: CompilerId,
}

impl ExportsInfoArtifactScopeGuard {
  fn enter(compiler_id: CompilerId, ptr: *mut ExportsInfoArtifact) -> Self {
    CURRENT_EXPORTS_INFO_ARTIFACT_STACK
      .lock()
      .expect("exports info artifact stack lock poisoned")
      .entry(compiler_id)
      .or_default()
      .push(ptr as usize);
    Self { compiler_id }
  }
}

impl Drop for ExportsInfoArtifactScopeGuard {
  fn drop(&mut self) {
    let mut lock = CURRENT_EXPORTS_INFO_ARTIFACT_STACK
      .lock()
      .expect("exports info artifact stack lock poisoned");
    let mut remove_entry = false;
    if let Some(stack) = lock.get_mut(&self.compiler_id) {
      let _ = stack.pop();
      remove_entry = stack.is_empty();
    }
    if remove_entry {
      lock.remove(&self.compiler_id);
    }
  }
}

pub(crate) fn current_exports_info_artifact(
  compiler_id: CompilerId,
) -> Option<*mut ExportsInfoArtifact> {
  CURRENT_EXPORTS_INFO_ARTIFACT_STACK
    .lock()
    .expect("exports info artifact stack lock poisoned")
    .get(&compiler_id)
    .and_then(|stack| stack.last().copied())
    .map(|ptr| ptr as *mut ExportsInfoArtifact)
}

pub(crate) async fn with_exports_info_artifact<F>(
  compiler_id: CompilerId,
  exports_info_artifact: &mut ExportsInfoArtifact,
  f: F,
) -> F::Output
where
  F: Future,
{
  let _guard = ExportsInfoArtifactScopeGuard::enter(compiler_id, exports_info_artifact as *mut _);
  f.await
}
