/*use std::{path::PathBuf, sync::Arc};

use rspack_cacheable::cacheable;

struct Option {
  root: PathBuf,
}

struct Context {
  options: Arc<Option>,
}

struct FromContext;

// impl S

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  option: Arc<Option>,
  name: String,
}

#[test]
fn test_as_ref_string() {
  let module = Module {
    module_type: ModuleType::NormalModule,
  };

  let bytes = rspack_cacheable::to_bytes(&module, &()).unwrap();
  let new_module: Module = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);
}*/
