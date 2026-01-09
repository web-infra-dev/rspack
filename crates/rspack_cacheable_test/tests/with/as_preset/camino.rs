use std::path::PathBuf;

use camino::Utf8PathBuf;
use rspack_cacheable::{
  CacheableContext, enable_cacheable as cacheable, from_bytes, to_bytes, with::AsPreset,
};

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  #[cacheable(with=AsPreset)]
  path: Utf8PathBuf,
}

struct Context {
  project_root: PathBuf,
}

impl CacheableContext for Context {
  fn project_root(&self) -> Option<&std::path::Path> {
    Some(&self.project_root)
  }
}

#[test]
fn test_preset_camino() {
  let module = Module {
    path: Utf8PathBuf::from("/home/user"),
  };

  let bytes = to_bytes(&module, &()).unwrap();
  let new_module: Module = from_bytes(&bytes, &()).unwrap();
  assert_eq!(module, new_module);

  // test with context
  let context = Context {
    project_root: PathBuf::from("/home"),
  };
  let bytes = to_bytes(&module, &context).unwrap();
  let new_module: Module = from_bytes(&bytes, &context).unwrap();
  assert_eq!(module, new_module);

  // test portable context
  let other_context = Context {
    project_root: PathBuf::from("/root"),
  };
  let bytes = to_bytes(&module, &context).unwrap();
  let new_module: Module = from_bytes(&bytes, &other_context).unwrap();
  assert_eq!(
    Module {
      path: Utf8PathBuf::from("/root/user")
    },
    new_module
  );
}
