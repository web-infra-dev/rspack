//! Invocation-scoped authority for portable loader output. Paths are transient, never interned.

use rspack_error::{Result, error};
use rspack_paths::Utf8PathBuf;

#[derive(Debug, Default)]
pub(crate) struct PortableOutput {
  log: Vec<Option<Vec<Utf8PathBuf>>>,
  // Outer None is idle; inner None is an open invocation without a declaration.
  current: Option<Option<Vec<Utf8PathBuf>>>,
}

/// Position before a context-bound loader cache chain executes.
#[derive(Debug, Clone, Copy)]
pub struct PortableOutputMark(usize);

impl PortableOutput {
  pub(crate) fn begin(&mut self) {
    debug_assert!(self.current.is_none());
    self.current = Some(None);
  }

  pub(crate) fn certify(&mut self, paths: Vec<Utf8PathBuf>) -> Result<()> {
    if paths.iter().any(|path| !path.is_absolute()) {
      return Err(error!("portable_output paths must be absolute"));
    }
    if let Some(current) = &mut self.current {
      current.get_or_insert_with(Vec::new).extend(paths);
    }
    Ok(())
  }

  pub(crate) fn end(&mut self) {
    let current = self
      .current
      .take()
      .expect("portable invocation must be open");
    self.log.push(current);
  }

  pub(crate) fn merge(&mut self, paths: Option<Vec<Utf8PathBuf>>) -> Result<()> {
    if paths
      .as_ref()
      .is_some_and(|paths| paths.iter().any(|path| !path.is_absolute()))
    {
      return Err(error!("portable_output paths must be absolute"));
    }
    self.log.push(paths);
    Ok(())
  }

  pub(crate) fn mark(&self) -> PortableOutputMark {
    PortableOutputMark(self.log.len())
  }

  pub(crate) fn since(&self, mark: PortableOutputMark) -> Option<Vec<Utf8PathBuf>> {
    let mut paths = Vec::new();
    for contribution in &self.log[mark.0..] {
      paths.extend(contribution.as_ref()?.iter().cloned());
    }
    paths.sort_unstable();
    paths.dedup();
    Some(paths)
  }

  pub(crate) fn into_paths(self) -> Option<Vec<Utf8PathBuf>> {
    let mut paths = Vec::new();
    for contribution in self.log {
      paths.extend(contribution?);
    }
    paths.sort_unstable();
    paths.dedup();
    Some(paths)
  }
}
