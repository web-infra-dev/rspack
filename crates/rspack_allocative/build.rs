fn main() {
  println!("cargo:rustc-check-cfg=cfg(rust_nightly)");
  println!("cargo:rustc-cfg=rust_nightly");
}
