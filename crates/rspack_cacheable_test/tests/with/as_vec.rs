use std::path::PathBuf;

use rspack_cacheable::{
  cacheable,
  with::{AsString, AsVec},
};
use rustc_hash::FxHashSet;

#[cacheable]
#[derive(Debug, PartialEq, Eq, Hash)]
struct Module {
  name: String,
}

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct App {
  #[with(AsVec)]
  modules: FxHashSet<Module>,
  #[with(AsVec<AsString>)]
  paths: Vec<PathBuf>,
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

  let app = App { modules, paths };

  let bytes = rspack_cacheable::to_bytes(&app, &mut ()).unwrap();
  let new_app: App = rspack_cacheable::from_bytes(&bytes, &mut ()).unwrap();
  assert_eq!(app, new_app);
}
