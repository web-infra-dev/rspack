mod alias;
mod browser_field;
mod builtins;
mod dependencies;
mod exports_field;
mod extension_alias;
mod extensions;
mod fallback;
mod full_specified;
mod imports_field;
mod incorrect_description_file;
mod main_field;
mod memory_fs;
mod missing;
#[cfg(not(target_os = "windows"))]
mod node_path;
mod package_json;
#[cfg(feature = "yarn_pnp")]
mod pnp;
mod resolve;
mod restrictions;
mod roots;
mod scoped_packages;
mod simple;
mod symlink;
mod tsconfig_paths;
mod tsconfig_project_references;
#[cfg(windows)]
mod windows;

use std::{env, path::PathBuf, sync::Arc, thread};

use crate::Resolver;

pub fn fixture_root() -> PathBuf {
  env::current_dir().unwrap().join("fixtures")
}

pub fn fixture() -> PathBuf {
  fixture_root()
    .join("enhanced_resolve")
    .join("test")
    .join("fixtures")
}

#[tokio::test]
async fn threaded_environment() {
  let cwd = env::current_dir().unwrap();
  let resolver = Arc::new(Resolver::default());
  for _ in 0..2 {
    _ = thread::spawn({
      let cwd = cwd.clone();
      let resolver = Arc::clone(&resolver);
      move || {
        _ = resolver.resolve(cwd, ".");
      }
    })
    .join();
  }
}
