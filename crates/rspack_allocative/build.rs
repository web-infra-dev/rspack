fn main() {
  println!("cargo:rerun-if-env-changed=RSPACK_ALLOCATIVE_DRIVER");
  if std::env::var_os("CARGO_CFG_ALLOCATIVE").is_some() {
    assert!(
      std::env::var("RSPACK_ALLOCATIVE_DRIVER").as_deref() == Ok("1"),
      "allocative builds require the compiler driver: pnpm exec zx scripts/allocative-driver.mjs cargo <arguments>"
    );
  }
}
