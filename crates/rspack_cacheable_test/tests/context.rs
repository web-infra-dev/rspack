use std::{any::Any, sync::Arc};

use rspack_cacheable::{
  Error, enable_cacheable as cacheable, from_bytes, to_bytes,
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

#[cacheable]
struct FromContext;

impl AsConverter<Arc<CompilerOptions>> for FromContext {
  fn serialize(_data: &Arc<CompilerOptions>, _ctx: &dyn Any) -> Result<Self, Error> {
    Ok(FromContext)
  }
  fn deserialize(self, ctx: &dyn Any) -> Result<Arc<CompilerOptions>, Error> {
    let Some(ctx) = ctx.downcast_ref::<Context>() else {
      return Err(Error::MessageError("context not match"));
    };
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

  assert!(matches!(
    from_bytes::<Module, ()>(&bytes, &()),
    Err(Error::MessageError("context not match"))
  ));
  let new_module: Module = from_bytes(&bytes, &context).unwrap();
  assert_eq!(module, new_module);
}
