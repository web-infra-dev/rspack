use std::path::PathBuf;

use dashmap::DashMap;
use rspack_cacheable::{
  cacheable,
  with::{AsCacheable, AsMap, AsString},
};
use rustc_hash::FxHashMap;

#[cacheable]
#[derive(Debug, PartialEq, Eq)]
struct Module {
  content: String,
}

#[cacheable]
#[derive(Debug)]
struct App {
  #[with(AsMap<AsCacheable, AsCacheable>)]
  modules: FxHashMap<String, Module>,
  #[with(AsMap<AsCacheable, AsString>)]
  paths: DashMap<String, PathBuf>,
}

#[test]
fn test_as_map() {
  let modules = FxHashMap::from_iter(vec![
    (
      String::from("file1"),
      Module {
        content: String::from("console.log('file')"),
      },
    ),
    (
      String::from("file2"),
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
