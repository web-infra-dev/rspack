use std::path::{Path, PathBuf};

pub fn resolve_from_context(context: &Path, template: &str) -> PathBuf {
  context.join(template)
}
