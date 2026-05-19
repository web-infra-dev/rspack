fn main() {
  // Registers `cargo::rerun-if-env-changed=NAPI_FORCE_BUILD_RSPACK_BINDING_API`
  // so cargo recompiles this crate (re-running the `#[napi]` proc macros that
  // emit type-def files into `NAPI_TYPE_DEF_TMP_FOLDER`) when the napi CLI bumps
  // that env var under `--no-dts-cache`. Without this, second-and-later builds
  // produce an empty `napi-binding.d.ts`.
  rspack_binding_build::setup();

  println!("cargo::rustc-check-cfg=cfg(tokio_unstable)");
}
