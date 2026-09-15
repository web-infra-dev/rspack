#![feature(unique_rc_arc)]

use std::{
  path::Path,
  sync::{Arc, UniqueArc},
};

use rspack_cacheable::{CacheableContext, from_bytes, to_bytes};
use rspack_collections::Identifiable;
use rspack_core::{BoxModule, Module, ModuleExt, ModuleRef, RawModule, RuntimeGlobals};
use rspack_util::source_map::SourceMapKind;

struct Context;

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&Path> {
    None
  }
}

fn raw_module() -> RawModule {
  RawModule::new(
    "module.exports = 42;".into(),
    "unique-module".into(),
    "unique-module".into(),
    RuntimeGlobals::empty(),
  )
}

fn assert_shared_allocation(mut module: BoxModule) {
  module.set_source_map_kind(SourceMapKind::SourceMap);
  let ptr = module.as_ref() as *const dyn Module;
  let shared = ModuleRef::from(module);
  assert!(std::ptr::eq(ptr, shared.as_ref()));
  assert_eq!(*shared.get_source_map_kind(), SourceMapKind::SourceMap);
  assert_eq!(shared.identifier(), "unique-module".into());

  let cloned = shared.clone();
  assert!(std::ptr::eq(shared.as_ref(), cloned.as_ref()));
  drop(shared);
  assert_eq!(cloned.identifier(), "unique-module".into());
}

#[test]
fn publish_preserves_module_allocation() {
  assert_shared_allocation(raw_module().boxed());

  let unique = UniqueArc::new(raw_module());
  let ptr = &*unique as *const RawModule;
  let module = BoxModule::new(unique);
  assert!(std::ptr::eq(
    ptr,
    module.downcast_ref::<RawModule>().expect("raw module")
  ));
  assert_shared_allocation(module);
}

#[test]
fn boxed_trait_object_can_be_published() {
  let module: Box<dyn Module> = Box::new(raw_module());
  assert_shared_allocation(module.into());
}

#[test]
fn arc_conversion_preserves_allocation_and_has_one_owner() {
  let module = raw_module().boxed();
  let ptr = module.as_ref() as *const dyn Module;
  let shared: Arc<dyn Module> = module.into();
  assert!(std::ptr::eq(ptr, Arc::as_ptr(&shared)));
  assert_eq!(Arc::strong_count(&shared), 1);
  assert_eq!(Arc::weak_count(&shared), 0);
}

#[test]
fn restored_module_can_be_mutated_and_published() {
  let module = raw_module().boxed();
  module.freeze_build_info();
  module.freeze_build_meta();
  let bytes = to_bytes(&module, &Context).expect("serialize module");
  let restored = from_bytes::<BoxModule, _>(&bytes, &Context).expect("deserialize module");
  assert!(restored.downcast_ref::<RawModule>().is_some());
  assert_shared_allocation(restored);
}
