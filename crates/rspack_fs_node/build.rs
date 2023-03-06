fn main() {
  napi_build::setup();

  // Rebuild binding options if and only if it's built for crate `node_binding`
  if std::env::var("OUT_DIR")
    .expect("should exist")
    .contains("node_binding")
  {
    println!("cargo:rerun-if-changed=../node_binding");
  }
}
