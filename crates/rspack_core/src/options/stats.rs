#[cfg(allocative)]
use rspack_util::allocative;

#[derive(Debug, Default)]
#[cfg_attr(allocative, derive(allocative::Allocative))]
pub struct StatsOptions {
  pub colors: bool,
}
