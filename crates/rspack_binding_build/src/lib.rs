pub fn setup() {
  napi_build::setup();
  println!("cargo:rustc-link-arg=-zstack-size=0x6400000");
}
