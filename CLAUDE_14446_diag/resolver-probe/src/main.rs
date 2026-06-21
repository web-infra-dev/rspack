// Probe for #14446: does rspack_resolver (the version rspack 2.0.8 uses)
// return a MIXED-separator path on Windows when the resolution base has a
// forward slash after the drive? realpath() rebuilds the path bottom-up via
// `real_path.push(segment)` (each push inserts the native `\`), seeded from a
// root whose separator comes from the input base — so a `D:/...` base should
// yield `D:/a\...\App.tsx`.
use std::{
  fs,
  path::{Path, PathBuf},
};

use rspack_resolver::{ResolveOptions, Resolver};

async fn show(label: &str, base: &Path, spec: &str, r: &Resolver) {
  match r.resolve(base, spec).await {
    Ok(res) => {
      let s = res.full_path().to_string_lossy().to_string();
      let mixed = s.contains('/') && s.contains('\\');
      println!("[{label}] base={base:?}\n    => {s}\n    mixed_sep={mixed}");
    }
    Err(e) => println!("[{label}] base={base:?} => ERR {e:?}"),
  }
}

#[tokio::main]
async fn main() {
  let tmp = std::env::var("RUNNER_TEMP").unwrap_or_else(|_| ".".into());
  let src = PathBuf::from(&tmp).join("resprobe").join("src");
  fs::create_dir_all(&src).unwrap();
  fs::write(src.join("App.tsx"), "x").unwrap();

  let opts = ResolveOptions {
    extensions: vec![".tsx".into(), ".ts".into(), ".js".into()],
    ..Default::default()
  };
  let r = Resolver::new(opts);

  let native = src.to_string_lossy().to_string();
  println!("native src string = {native}");
  println!("std::canonicalize  = {:?}\n", fs::canonicalize(&src));

  // 1) native base (what cwd/dirname normally gives)
  show("native-base", &src, "./App", &r).await;

  // 2) fully forward-slash base
  let fully_forward = native.replace('\\', "/");
  show("forward-base", Path::new(&fully_forward), "./App", &r).await;

  // 3) drive-only-forward base: `D:/a\...\src` (forward only right after drive)
  let drive_fwd = if native.len() > 3 && native.as_bytes()[1] == b':' {
    format!("{}:/{}", &native[0..1], &native[3..])
  } else {
    native.clone()
  };
  show("drive-forward-base", Path::new(&drive_fwd), "./App", &r).await;
}
