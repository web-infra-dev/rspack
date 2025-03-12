use std::{env, path::PathBuf, process::Command};

fn main() {
  let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
  let mut go_build = Command::new("go");

  go_build
    .arg("build")
    .arg("-v")
    .arg("-buildmode=c-archive")
    .arg("-o")
    .arg(out_dir.join("libgo.a"))
    .arg("./binding/wrapper.go");

  go_build.status().expect("Go build failed");

  println!("cargo:rerun-if-changed={}", "./binding/wrapper.go");
  println!(
    "cargo:rustc-link-search=native={}",
    out_dir.to_str().unwrap()
  );
  println!("cargo:rustc-link-lib=static=go");
}
