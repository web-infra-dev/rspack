///! See documentation at <https://docs.rs/rspack_resolver>
use std::{env, path::PathBuf};

use rspack_resolver::{AliasValue, ResolveOptions, Resolver};

#[tokio::main]
async fn main() {
  let path = PathBuf::from(env::args().nth(1).expect("path"));

  assert!(
    path.is_dir(),
    "{path:?} must be a directory that will be resolved against."
  );
  assert!(path.is_absolute(), "{path:?} must be an absolute path.",);

  let specifier = env::args().nth(2).expect("specifier");

  println!("path: {path:?}");
  println!("specifier: {specifier}");

  let options = ResolveOptions {
    alias_fields: vec![vec!["browser".into()]],
    alias: vec![("asdf".into(), vec![AliasValue::from("./test.js")])],
    extensions: vec![".js".into(), ".ts".into()],
    extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
    // ESM
    condition_names: vec!["node".into(), "import".into()],
    // CJS
    // condition_names: vec!["node".into(), "require".into()],
    ..ResolveOptions::default()
  };
  let mut ctx = Default::default();

  match Resolver::new(options)
    .resolve_with_context(path, &specifier, &mut ctx)
    .await
  {
    Err(error) => println!("Error: {error}"),
    Ok(resolution) => println!("Resolved: {:?}", resolution.full_path()),
  };

  let mut sorted_file_deps = ctx.file_dependencies.iter().collect::<Vec<_>>();
  sorted_file_deps.sort_by_key(|p| p.as_path());
  println!("file_deps: {:#?}", sorted_file_deps);

  let mut sorted_missing = ctx.missing_dependencies.iter().collect::<Vec<_>>();
  sorted_missing.sort_by_key(|p| p.as_path());
  println!("missing_deps: {:#?}", sorted_missing);
}
