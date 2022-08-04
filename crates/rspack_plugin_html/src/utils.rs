use std::path::{Path, PathBuf};

pub fn resolve_from_context(context: &str, template: &str) -> PathBuf {
  Path::new(context).join(template)
}
