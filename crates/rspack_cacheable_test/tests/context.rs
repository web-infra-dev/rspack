use std::sync::Arc;

use rspack_cacheable::{
  CacheableContext, ContextGuard, Error, Result, enable_cacheable as cacheable, from_bytes,
  to_bytes,
  with::{As, AsConverter},
};

#[derive(Debug, PartialEq, Eq)]
struct CompilerOptions {
  data: usize,
}

#[derive(Debug)]
struct Context {
  option: Arc<CompilerOptions>,
}

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&std::path::Path> {
    None
  }
}

#[cacheable]
struct FromContext;

impl AsConverter<Arc<CompilerOptions>> for FromContext {
  fn serialize(_data: &Arc<CompilerOptions>, _guard: &ContextGuard) -> Result<Self> {
    Ok(FromContext)
  }
  fn deserialize(self, guard: &ContextGuard) -> Result<Arc<CompilerOptions>> {
    let ctx = guard.downcast_context::<Context>()?;
    Ok(ctx.option.clone())
  }
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=As<FromContext>)]
  compiler_option: Arc<CompilerOptions>,
  name: String,
}

#[test]
fn test_context() {
  let context = Context {
    option: Arc::new(CompilerOptions { data: 1 }),
  };
  let module = Module {
    compiler_option: context.option.clone(),
    name: String::from("a"),
  };

  let bytes = to_bytes(&module, &()).unwrap();

  let result = from_bytes::<Module, ()>(&bytes, &());
  assert!(result.is_err(), "should fail when using wrong context");
  if let Err(e) = result {
    assert!(
      matches!(e, Error::NoContext),
      "expected NoContext but got: {:?}",
      e
    );
  }
  let new_module: Module = from_bytes(&bytes, &context).unwrap();
  assert_eq!(module, new_module);
}
