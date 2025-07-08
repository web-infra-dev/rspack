// Run `cargo codegen` to generate the file.
mod generated;

pub use generated::{rspack_pkg_version, rspack_swc_core_version};

#[macro_export]
macro_rules! rspack_pkg_version {
  () => {
    $crate::rspack_pkg_version()
  };
}

#[macro_export]
macro_rules! rspack_swc_core_version {
  () => {
    $crate::rspack_swc_core_version()
  };
}
