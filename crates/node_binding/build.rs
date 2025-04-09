fn main() {
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");
  napi_build::setup();
}
