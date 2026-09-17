use std::sync::{
  Arc, OnceLock, Weak,
  atomic::{AtomicBool, AtomicUsize, Ordering},
};

use rspack::builder::Builder;
use rspack_core::{
  AssetEmittedInfo, CacheOptions, Compilation, Compiler, CompilerAfterEmit, CompilerAssetEmitted,
  CompilerEmit, Mode, Plugin,
};
use rspack_error::{Result, error};
use rspack_fs::{MemoryFileSystem, WritableFileSystem};
use rspack_hook::{plugin, plugin_hook};

fn compiler(fs: Arc<MemoryFileSystem>, probe: Option<OwnershipProbe>) -> Compiler {
  Compiler::builder()
    .context("/")
    .entry("main", "/src/index.js")
    .mode(Mode::Development)
    .cache(CacheOptions::Disabled)
    .plugins(probe.map(|probe| Box::new(probe) as rspack_core::BoxPlugin))
    .input_filesystem(fs.clone())
    .output_filesystem(fs)
    .build()
    .expect("create compiler")
}

// Business functions keep borrowing the compilation; only the caller that
// owns the pointer transfers it into and out of a shared scheduling phase.
fn mutate(compilation: &mut Compilation) {
  compilation.hot_index += 1;
}

#[tokio::test]
async fn owned_handoff_preserves_allocation_and_existing_mutable_api() {
  let mut compiler = compiler(Arc::new(MemoryFileSystem::default()), None);
  let weak = compiler.compilation.downgrade();
  let pointer = weak.as_ptr();
  mutate(&mut compiler.compilation);
  assert!(weak.upgrade().is_none());

  let shared = compiler.compilation.share();
  let reader = Arc::clone(&shared);
  assert!(compiler.compilation.try_make_unique().is_err());
  assert!(
    compiler
      .build()
      .await
      .expect_err("build must reject a retained reader")
      .to_string()
      .contains("Arc readers")
  );
  assert_eq!(compiler.compilation.hot_index, 1);
  let observed = tokio::spawn(async move { reader.hot_index })
    .await
    .expect("reader task completes");
  assert_eq!(observed, 1);
  assert_eq!(Arc::as_ptr(&shared), pointer);

  drop(shared);
  compiler
    .compilation
    .try_make_unique()
    .expect("all readers have finished");
  assert!(weak.upgrade().is_none());
  mutate(&mut compiler.compilation);
  assert_eq!(compiler.compilation.hot_index, 2);
  assert_eq!(&*compiler.compilation as *const Compilation, pointer);
  drop(compiler);
  assert!(weak.upgrade().is_none());
}

#[tokio::test]
async fn build_and_incremental_rebuild_preserve_the_compilation_address() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/src".into())
    .await
    .expect("create sources");
  fs.write("/src/index.js".into(), b"console.log('first');")
    .await
    .expect("write source");
  let weak_slot = Arc::new(OnceLock::new());
  let events = Arc::new(AtomicUsize::new(0));
  let probe = OwnershipProbe::new_inner(
    Arc::clone(&weak_slot),
    Arc::clone(&events),
    Arc::new(AtomicBool::new(false)),
  );
  let mut compiler = compiler(fs.clone(), Some(probe));
  weak_slot
    .set(compiler.compilation.downgrade())
    .expect("set owner");
  let weak = compiler.compilation.downgrade();
  let pointer = weak.as_ptr();

  compiler.build().await.expect("initial build");
  assert_eq!(compiler.compilation.get_errors().count(), 0);
  assert_eq!(events.swap(0, Ordering::Relaxed), 7);
  let initial_id = compiler.compilation.id();
  assert_eq!(&*compiler.compilation as *const Compilation, pointer);
  assert!(weak.upgrade().is_none());

  fs.write("/src/index.js".into(), b"console.log('second');")
    .await
    .expect("update source");
  compiler
    .rebuild(
      ["/src/index.js".to_owned()].into_iter().collect(),
      Default::default(),
    )
    .await
    .expect("incremental rebuild");
  assert_eq!(compiler.compilation.get_errors().count(), 0);
  assert_eq!(events.load(Ordering::Relaxed), 7);
  assert_ne!(compiler.compilation.id(), initial_id);
  assert!(!compiler.compilation.assets().is_empty());
  assert_eq!(&*compiler.compilation as *const Compilation, pointer);
  assert!(weak.upgrade().is_none());

  let shared = compiler.compilation.share();
  let reader = weak.upgrade().expect("published compilation is readable");
  assert!(Arc::ptr_eq(&reader, &shared));
  assert_ne!(reader.id(), initial_id);
}

#[plugin]
#[derive(Debug)]
struct OwnershipProbe {
  weak: Arc<OnceLock<Weak<Compilation>>>,
  events: Arc<AtomicUsize>,
  fail: Arc<AtomicBool>,
}

#[plugin_hook(CompilerEmit for OwnershipProbe)]
async fn before_emit(&self, _compilation: &mut Compilation) -> Result<()> {
  assert!(
    self
      .weak
      .get()
      .expect("owner configured")
      .upgrade()
      .is_none()
  );
  self.events.fetch_or(1, Ordering::Relaxed);
  Ok(())
}

#[plugin_hook(CompilerAssetEmitted for OwnershipProbe)]
async fn asset_emitted(
  &self,
  compilation: &Compilation,
  _filename: &str,
  _info: &AssetEmittedInfo,
) -> Result<()> {
  let reader = self
    .weak
    .get()
    .expect("owner configured")
    .upgrade()
    .expect("shared emit phase");
  assert!(std::ptr::eq(Arc::as_ptr(&reader), compilation));
  self.events.fetch_or(2, Ordering::Relaxed);
  if self.fail.load(Ordering::Relaxed) {
    return Err(error!("intentional emit failure"));
  }
  Ok(())
}

#[plugin_hook(CompilerAfterEmit for OwnershipProbe)]
async fn after_emit(&self, _compilation: &mut Compilation) -> Result<()> {
  assert!(
    self
      .weak
      .get()
      .expect("owner configured")
      .upgrade()
      .is_none()
  );
  self.events.fetch_or(4, Ordering::Relaxed);
  Ok(())
}

impl Plugin for OwnershipProbe {
  fn name(&self) -> &'static str {
    "OwnershipProbe"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.emit.tap(before_emit::new(self));
    ctx
      .compiler_hooks
      .asset_emitted
      .tap(asset_emitted::new(self));
    ctx.compiler_hooks.after_emit.tap(after_emit::new(self));
    Ok(())
  }
}

#[tokio::test]
async fn emit_failure_restores_unique_ownership_before_retry() {
  let fs = Arc::new(MemoryFileSystem::default());
  fs.create_dir_all("/src".into())
    .await
    .expect("create sources");
  fs.write("/src/index.js".into(), b"console.log('hello');")
    .await
    .expect("write source");
  let weak_slot = Arc::new(OnceLock::new());
  let events = Arc::new(AtomicUsize::new(0));
  let fail = Arc::new(AtomicBool::new(true));
  let probe = OwnershipProbe::new_inner(
    Arc::clone(&weak_slot),
    Arc::clone(&events),
    Arc::clone(&fail),
  );
  let mut compiler = compiler(fs, Some(probe));
  weak_slot
    .set(compiler.compilation.downgrade())
    .expect("set owner");
  assert!(compiler.build().await.is_err());
  assert_eq!(events.swap(0, Ordering::Relaxed), 3);
  assert!(
    weak_slot
      .get()
      .expect("owner configured")
      .upgrade()
      .is_none()
  );
  mutate(&mut compiler.compilation);

  fail.store(false, Ordering::Relaxed);
  compiler.build().await.expect("retry build");
  assert_eq!(events.load(Ordering::Relaxed), 7);
}
