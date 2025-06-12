fn main() {
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");
  rspack_node_build::setup();
}
