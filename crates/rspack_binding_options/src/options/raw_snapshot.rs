use napi_derive::napi;
use rspack_core::{CompilerOptionsBuilder, SnapshotOptions, SnapshotStrategy};
use serde::Deserialize;

use crate::RawOption;

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSnapshotStrategy {
  pub hash: bool,
  pub timestamp: bool,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
#[napi(object)]
pub struct RawSnapshotOptions {
  pub resolve_build_dependencies: RawSnapshotStrategy,
  pub build_dependencies: RawSnapshotStrategy,
  pub resolve: RawSnapshotStrategy,
  pub module: RawSnapshotStrategy,
}

impl RawOption<SnapshotOptions> for RawSnapshotOptions {
  fn to_compiler_option(
    self,
    _options: &CompilerOptionsBuilder,
  ) -> anyhow::Result<SnapshotOptions> {
    let RawSnapshotOptions {
      resolve_build_dependencies:
        RawSnapshotStrategy {
          hash: a,
          timestamp: b,
        },
      build_dependencies: RawSnapshotStrategy {
        hash: c,
        timestamp: d,
      },
      resolve: RawSnapshotStrategy {
        hash: e,
        timestamp: f,
      },
      module: RawSnapshotStrategy {
        hash: g,
        timestamp: h,
      },
    } = self;

    Ok(SnapshotOptions {
      resolve_build_dependencies: SnapshotStrategy {
        hash: a,
        timestamp: b,
      },
      build_dependencies: SnapshotStrategy {
        hash: c,
        timestamp: d,
      },
      resolve: SnapshotStrategy {
        hash: e,
        timestamp: f,
      },
      module: SnapshotStrategy {
        hash: g,
        timestamp: h,
      },
    })
  }

  fn fallback_value(_: &CompilerOptionsBuilder) -> Self {
    Default::default()
  }
}
