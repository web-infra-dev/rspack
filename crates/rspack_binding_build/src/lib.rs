pub fn setup() {
  napi_build::setup();
  println!("cargo:rustc-link-arg=-zstack-size=64000000");
}
