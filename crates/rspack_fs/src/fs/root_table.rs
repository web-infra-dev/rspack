use std::path::PathBuf;

use rustc_hash::FxHashMap;

use super::ReadableFileSystem;
use crate::{
  AbsPath, Error, ProjectPath, ProjectPathParts, Result,
  path::normalize::{PathFlavor, abs_strip_prefix, normalize_abs_path, validate_root_name},
};

#[derive(Debug)]
struct RootEntry {
  name: Box<str>,
  root: AbsPath,
  canonical_root: Option<AbsPath>,
}

/// Flat, non-overlapping table of named logical roots (`project`, `gvs`, `@extra`, ...).
#[derive(Debug, Default)]
pub struct RootTable {
  entries: Box<[RootEntry]>,
  /// `(prefix, entry_index)` pairs sorted by descending byte length.
  prefixes: Box<[(AbsPath, usize)]>,
  by_name: FxHashMap<Box<str>, usize>,
}

impl RootTable {
  pub fn from_fs(fs: &dyn ReadableFileSystem, roots: Vec<(String, AbsPath)>) -> Result<Self> {
    Self::build(roots, |root| fs.canonicalize_sync(root).ok())
  }

  pub fn without_canonicalization(roots: Vec<(String, AbsPath)>) -> Result<Self> {
    Self::build(roots, |_| None)
  }

  fn build(
    roots: Vec<(String, AbsPath)>,
    mut canonicalize_fn: impl FnMut(&AbsPath) -> Option<AbsPath>,
  ) -> Result<Self> {
    let mut entries = Vec::with_capacity(roots.len());
    let mut by_name = FxHashMap::with_capacity_and_hasher(roots.len(), Default::default());

    for (idx, (name, root)) in roots.into_iter().enumerate() {
      validate_root_name(&name)?;
      let name: Box<str> = name.into_boxed_str();
      if by_name.insert(name.clone(), idx).is_some() {
        return Err(Error::InvalidRootName(name.into_string()));
      }
      let canonical_root = canonicalize_fn(&root).filter(|canon| canon != &root);
      entries.push(RootEntry {
        name,
        root,
        canonical_root,
      });
    }

    // Reject any overlap across distinct logical/canonical root entries so encoding is unambiguous.
    for (i, left) in entries.iter().enumerate() {
      for right in &entries[i + 1..] {
        for lhs in [Some(&left.root), left.canonical_root.as_ref()]
          .into_iter()
          .flatten()
        {
          for rhs in [Some(&right.root), right.canonical_root.as_ref()]
            .into_iter()
            .flatten()
          {
            if lhs.strip_prefix(rhs).is_some() || rhs.strip_prefix(lhs).is_some() {
              return Err(Error::OverlappingRoot {
                first_name: left.name.to_string(),
                first_path: lhs.as_str().to_string(),
                second_name: right.name.to_string(),
                second_path: rhs.as_str().to_string(),
              });
            }
          }
        }
      }
    }

    let mut prefixes = Vec::with_capacity(entries.len() * 2);
    for (idx, entry) in entries.iter().enumerate() {
      prefixes.push((entry.root.clone(), idx));
      if let Some(canon) = &entry.canonical_root {
        prefixes.push((canon.clone(), idx));
      }
    }
    prefixes.sort_by_key(|(a, _)| std::cmp::Reverse(a.as_str().len()));

    Ok(Self {
      entries: entries.into_boxed_slice(),
      prefixes: prefixes.into_boxed_slice(),
      by_name,
    })
  }

  #[inline]
  pub fn is_empty(&self) -> bool {
    self.entries.is_empty()
  }

  #[inline]
  pub fn len(&self) -> usize {
    self.entries.len()
  }

  pub fn root_names(&self) -> Vec<&str> {
    let mut names: Vec<&str> = self.entries.iter().map(|e| &*e.name).collect();
    names.sort_unstable();
    names
  }

  #[inline]
  pub fn root_abs_path(&self, name: &str) -> Option<&AbsPath> {
    self.by_name.get(name).map(|&idx| &self.entries[idx].root)
  }

  pub(crate) fn to_project_path_normalized_str(&self, normalized_abs: &str) -> ProjectPath {
    for (prefix, idx) in &self.prefixes {
      if let Some(rel) = abs_strip_prefix(normalized_abs, prefix.as_str(), PathFlavor::HOST)
        && (cfg!(windows) || !rel.contains('\\'))
      {
        return ProjectPath::from_validated_root_and_os_rel(&self.entries[*idx].name, rel);
      }
    }
    ProjectPath::external_from_normalized_str(normalized_abs)
  }

  pub fn to_project_path_str(&self, raw: &str) -> Result<ProjectPath> {
    let normalized = normalize_abs_path(raw, PathFlavor::HOST)?;
    Ok(self.to_project_path_normalized_str(normalized.as_ref()))
  }

  #[inline]
  pub fn to_project_path_utf8(&self, path: &camino::Utf8Path) -> Result<ProjectPath> {
    self.to_project_path_str(path.as_str())
  }

  #[inline]
  pub fn to_utf8_path_buf(&self, path: &ProjectPath) -> Result<camino::Utf8PathBuf> {
    Ok(self.to_abs_path(path)?.to_utf8_path_buf())
  }

  #[inline]
  pub fn to_std_path_buf(&self, path: &ProjectPath) -> Result<PathBuf> {
    Ok(self.to_abs_path(path)?.to_std_path_buf())
  }

  pub fn to_project_path(&self, abs: &AbsPath) -> ProjectPath {
    for (prefix, idx) in &self.prefixes {
      if let Some(rel) = abs.strip_prefix(prefix)
        && (cfg!(windows) || !rel.contains('\\'))
      {
        return ProjectPath::from_validated_root_and_os_rel(&self.entries[*idx].name, rel);
      }
    }
    ProjectPath::external(abs)
  }

  pub fn to_abs_path(&self, path: &ProjectPath) -> Result<AbsPath> {
    match path.parts() {
      ProjectPathParts::Anchored { root, rel } => {
        let Some(&idx) = self.by_name.get(root) else {
          return Err(Error::UnknownRoot(root.to_string()));
        };
        Ok(self.entries[idx].root.join_normalized_posix_rel(rel))
      }
      ProjectPathParts::External(abs) => {
        if abs.starts_with('/') == matches!(PathFlavor::HOST, PathFlavor::Unix) {
          Ok(AbsPath::from_normalized(abs))
        } else {
          Err(Error::NotAbsolutePath(abs.to_string()))
        }
      }
    }
  }
}
