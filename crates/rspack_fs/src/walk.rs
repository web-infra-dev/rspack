use std::borrow::Cow;

use rspack_paths::{Utf8Path, Utf8PathBuf};
use rspack_util::glob::GlobPattern;

use crate::{ReadableFileSystem, Result};

/// The matching state carried while walking a directory.
pub trait WalkMatcher: Clone + Send + Sync {
  /// Treat an entry as a file if its metadata cannot be read.
  const VISIT_ON_METADATA_ERROR: bool = false;

  /// Prepare the state for an entry before its type is known. Return `None`
  /// only if the entry and any descendants can be skipped.
  fn prepare_child(&self, _dir: &Utf8Path, _name: &str) -> Option<Self> {
    Some(self.clone())
  }

  /// Return the child matcher to descend into a directory or yield a file.
  fn match_entry(self, _dir: &Utf8Path, _name: &str, _is_directory: bool) -> Option<Self> {
    Some(self)
  }

  /// Walk a directory tree depth-first. Pass each matched file path and its
  /// child matcher to the callback.
  fn walk<'a>(
    &'a self,
    root: &'a Utf8Path,
    fs: &'a dyn ReadableFileSystem,
    skip_dotfiles: bool,
    on_file: &'a mut (impl FnMut(Utf8PathBuf, Self) + Send),
  ) -> impl Future<Output = Result<()>> + Send + 'a {
    async move {
      if !fs
        .metadata(root)
        .await
        .is_ok_and(|metadata| metadata.is_directory)
      {
        return Ok(());
      }
      let names = fs.read_dir(root).await?;
      // Suspend the parent iterator and state to preserve depth-first order.
      let mut stack = vec![(root.to_path_buf(), Cow::Borrowed(self), names.into_iter())];
      while let Some((root, matcher, names)) = stack.last_mut() {
        let Some(name) = names.next() else {
          stack.pop();
          continue;
        };
        let Some(child) = matcher.prepare_child(root, &name) else {
          continue;
        };
        let path = root.join(&name);
        let is_directory = match fs.metadata(&path).await {
          Ok(metadata) if metadata.is_directory => true,
          Err(_) if !Self::VISIT_ON_METADATA_ERROR => continue,
          _ if skip_dotfiles && name.starts_with('.') => continue,
          _ => false,
        };
        let Some(child) = child.match_entry(root, &name, is_directory) else {
          continue;
        };
        if is_directory {
          if fs
            .metadata(&path)
            .await
            .is_ok_and(|metadata| metadata.is_directory)
          {
            let names = fs.read_dir(&path).await?;
            stack.push((path, Cow::Owned(child), names.into_iter()));
          }
        } else {
          on_file(path, child);
        }
      }
      Ok(())
    }
  }
}

impl WalkMatcher for GlobPattern<'_> {
  const VISIT_ON_METADATA_ERROR: bool = true;

  fn prepare_child(&self, _dir: &Utf8Path, name: &str) -> Option<Self> {
    self.match_component(name)
  }

  fn match_entry(self, _dir: &Utf8Path, _name: &str, is_directory: bool) -> Option<Self> {
    if is_directory {
      self.match_prefix("/")
    } else {
      self.is_match("").then_some(self)
    }
  }
}
