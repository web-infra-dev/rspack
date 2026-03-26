fn main() {
  println!("cargo::rerun-if-env-changed=NAPI_TYPE_DEF_TMP_FOLDER");
  let crate_name = std::env::var("CARGO_PKG_NAME").expect("CARGO_PKG_NAME is not set");
  let force_build_env = crate_name
    .chars()
    .map(|ch| match ch {
      '-' => '_',
      _ => ch.to_ascii_uppercase(),
    })
    .collect::<String>();
  println!("cargo::rerun-if-env-changed=NAPI_FORCE_BUILD_{force_build_env}");
  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");
}
