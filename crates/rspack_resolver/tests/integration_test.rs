//! Test public APIs

use std::{env, path::PathBuf};

use rspack_resolver::{
  EnforceExtension, ModuleType, Resolution, ResolveContext, ResolveOptions, Resolver,
};

fn dir() -> PathBuf {
  env::current_dir().expect("current dir should be accessible")
}

async fn resolve(specifier: &str) -> Resolution {
  let path = dir();
  Resolver::new(ResolveOptions::default())
    .resolve(path, specifier)
    .await
    .expect("specifier should resolve")
}

#[tokio::test]
async fn clone() {
  let resolution = resolve("./tests/package.json").await;
  assert_eq!(resolution.clone(), resolution);
}

#[tokio::test]
async fn debug() {
  let resolution = resolve("./tests/package.json").await;
  let s = format!("{resolution:?}");
  assert!(!s.is_empty());
}

#[tokio::test]
async fn eq() {
  let resolution = resolve("./tests/package.json").await;
  assert_eq!(resolution, resolution);
}

#[tokio::test]
async fn package_json() {
  let resolution = resolve("./tests/package.json").await;
  let package_json = resolution.package_json().unwrap();
  assert_eq!(package_json.name.as_ref().unwrap(), "name");
  assert_eq!(package_json.r#type, Some(ModuleType::Module));
  assert!(package_json.side_effects.is_some());
}

#[cfg(feature = "package_json_raw_json_api")]
#[tokio::test]
async fn package_json_raw_json_api() {
  let resolution = resolve("./tests/package.json").await;
  assert!(
    resolution
      .package_json()
      .unwrap()
      .raw_json()
      .get("name")
      .is_some_and(|name| name == "name")
  );
}

#[tokio::test]
async fn clear_cache() {
  let resolver = Resolver::new(ResolveOptions::default());
  resolver.clear_cache(); // exists
}

#[tokio::test]
async fn options() {
  let resolver = Resolver::new(ResolveOptions::default());
  let options = resolver.options();
  assert!(!format!("{options:?}").is_empty());
}

#[tokio::test]
async fn debug_resolver() {
  let resolver = Resolver::new(ResolveOptions::default());
  assert!(!format!("{resolver:?}").is_empty());
}

#[tokio::test]
async fn dependencies() {
  let path = dir();
  let mut ctx = ResolveContext::default();
  let _ = Resolver::new(ResolveOptions::default())
    .resolve_with_context(path, "./tests/package.json", &mut ctx)
    .await;
  assert!(!ctx.file_dependencies.is_empty());
  assert!(ctx.missing_dependencies.is_empty());
}

#[tokio::test]
async fn options_api() {
  _ = ResolveOptions::default()
    .with_builtin_modules(true)
    .with_condition_names(&[])
    .with_extension(".js")
    .with_force_extension(EnforceExtension::Auto)
    .with_fully_specified(true)
    .with_main_field("asdf")
    .with_main_file("main")
    .with_module("module")
    .with_prefer_absolute(true)
    .with_prefer_relative(true)
    .with_root(PathBuf::new())
    .with_symbolic_link(true);
}

#[cfg(windows)]
#[tokio::test]
async fn windows_dos_device_paths() {
  let tests_dir = dir().join("tests");
  let tests_dir = tests_dir
    .to_str()
    .expect("resolver test directory should be UTF-8");
  let contexts = [
    PathBuf::from(format!(r"\\?\{tests_dir}")),
    PathBuf::from(format!(r"\\.\{tests_dir}")),
  ];
  let resolver = Resolver::new(ResolveOptions {
    symlinks: false,
    ..ResolveOptions::default()
  });

  for context in contexts {
    let expected = context.join("package.json");
    let resolution = resolver
      .resolve(&context, "./package.json")
      .await
      .expect("relative request should resolve from a DOS device path");
    assert_eq!(resolution.path(), expected.as_path());

    let request = format!("{}?foo=bar#fragment", expected.display());
    let resolution = resolver
      .resolve(dir(), &request)
      .await
      .expect("absolute DOS device path should resolve");
    assert_eq!(resolution.path(), expected.as_path());
    assert_eq!(resolution.query(), Some("?foo=bar"));
    assert_eq!(resolution.fragment(), Some("#fragment"));
  }
}
