// Run `cargo codegen` to generate the file.
#[doc(hidden)]
pub mod generated;

/// The version of the JavaScript `@rspack/core` package.
#[macro_export]
macro_rules! rspack_pkg_version {
  () => {
    $crate::generated::rspack_pkg_version()
  };
}

/// The version of the Rust workspace in the root `Cargo.toml` of the repository.
#[macro_export]
macro_rules! rspack_workspace_version {
  () => {
    $crate::generated::rspack_workspace_version()
  };
}

/// The version of the `swc_core` package used in the current workspace.
#[macro_export]
macro_rules! rspack_swc_core_version {
  () => {
    $crate::generated::rspack_swc_core_version()
  };
}
