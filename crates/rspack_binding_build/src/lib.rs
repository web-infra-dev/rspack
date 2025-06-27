pub fn setup() {
  napi_build::setup();
  #[cfg(target_family = "wasm")]
  println!("cargo:rustc-link-arg=-zstack-size=64000000");
}
