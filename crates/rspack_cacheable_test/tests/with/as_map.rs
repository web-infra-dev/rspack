use std::path::PathBuf;

use dashmap::DashMap;
use rspack_cacheable::{
  enable_cacheable as cacheable,
  utils::PortablePath,
  with::{As, AsCacheable, AsMap},
};
use rustc_hash::FxHashMap;

#[cacheable(hashable)]
#[derive(Debug, PartialEq, Eq, Hash)]
struct ModuleId(String);

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  content: String,
}

#[cacheable]
#[derive(Debug)]
struct App {
  modules: FxHashMap<ModuleId, Module>,
  #[cacheable(with=AsMap<AsCacheable, As<PortablePath>>)]
  paths: DashMap<String, PathBuf>,
}

#[test]
fn test_as_map() {
  let modules = FxHashMap::from_iter(vec![
    (
      ModuleId(String::from("file1")),
      Module {
        content: String::from("console.log('file')"),
      },
    ),
    (
      ModuleId(String::from("file2")),
      Module {
        content: String::from("export const file2 = 1;"),
      },
    ),
  ]);
  let paths = dashmap::DashMap::from_iter(vec![
    (String::from("file1"), PathBuf::from("/a")),
    (String::from("file2"), PathBuf::from("/a")),
  ]);
  let app = App { modules, paths };

  let bytes = rspack_cacheable::to_bytes(&app, &()).unwrap();
  let new_app: App = rspack_cacheable::from_bytes(&bytes, &()).unwrap();
  assert_eq!(app.modules, new_app.modules);
  assert_eq!(app.paths.len(), new_app.paths.len());
  for item in app.paths.iter() {
    assert_eq!(
      *item.value(),
      *new_app.paths.get(item.key()).expect("should have app path")
    );
  }
}
