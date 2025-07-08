mod generated;

pub use generated::{rspack_swc_core_version, rspack_version};

#[macro_export]
macro_rules! rspack_version {
  () => {
    $crate::rspack_version()
  };
}

#[macro_export]
macro_rules! rspack_swc_core_version {
  () => {
    $crate::rspack_swc_core_version()
  };
}
