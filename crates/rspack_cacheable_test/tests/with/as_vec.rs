use std::path::PathBuf;

use rspack_cacheable::{
  enable_cacheable as cacheable,
  utils::PortablePath,
  with::{As, AsVec},
};
use rustc_hash::FxHashSet;

#[cacheable(hashable)]
#[derive(Debug, PartialEq, Eq, Hash)]
struct Module {
  name: String,
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct App {
  modules: FxHashSet<Module>,
  #[cacheable(with=AsVec<As<PortablePath>>)]
  paths: Vec<PathBuf>,
  sizes: Vec<u32>,
}

#[test]
fn test_as_vec() {
  let modules = FxHashSet::from_iter(vec![
    Module {
      name: String::from("a"),
    },
    Module {
      name: String::from("b"),
    },
  ]);
  let paths = vec![PathBuf::from("/a"), PathBuf::from("/b")];
  let sizes = vec![1, 2];

  let app = App {
    modules,
    paths,
    sizes,
  };

  let bytes = rspack_cacheable::to_bytes(&app, &()).unwrap();
  let new_app: App = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(app, new_app);
}
