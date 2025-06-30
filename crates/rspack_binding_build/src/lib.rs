pub fn setup() {
  napi_build::setup();

  // Remove this after releasing https://github.com/napi-rs/napi-rs/pull/2748
  #[cfg(target_family = "wasm")]
  println!("cargo:rustc-link-arg=-zstack-size=64000000");
}
