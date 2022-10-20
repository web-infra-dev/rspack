use napi::bindgen_prelude::*;

#[napi]
pub enum DiffStatKind {
  Changed,
  Deleted,
  Added,
}

impl From<u8> for DiffStatKind {
  fn from(n: u8) -> Self {
    match n {
      0 => Self::Changed,
      1 => Self::Deleted,
      2 => Self::Added,
      _ => unreachable!(),
    }
  }
}

// TODO: remove it after hash
#[napi]
pub struct DiffStat {
  pub content: String,
  pub kind: DiffStatKind,
}

#[napi(object)]
pub struct RspackError {
  pub message: String,
}

#[napi(object)]
pub struct Stats {
  pub errors: Vec<RspackError>,
}

impl<'a> From<rspack_core::Stats<'a>> for Stats {
  fn from(rspack_stats: rspack_core::Stats) -> Self {
    Self {
      errors: rspack_stats
        .compilation
        .diagnostic
        .iter()
        .map(|d| RspackError {
          message: d.message.clone(),
        })
        .collect(),
    }
  }
}
