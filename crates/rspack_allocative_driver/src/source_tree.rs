use std::{
  fs, io,
  path::{Path, PathBuf},
};

use rustc_hash::{FxHashMap, FxHashSet};
use syn::{ItemMod, visit::Visit};

use crate::{Imports, ManualAdapters, instrument_file};

struct Source {
  path: PathBuf,
  module: Vec<String>,
  text: String,
  parsed: syn::File,
}

/// Collect manual adapters across this compilation target's source modules
/// before adding any derives. Other targets and generated files are excluded.
pub fn instrument_source_tree(
  root: &Path,
  package: &Path,
  generated: Option<&Path>,
  crate_path: &str,
) -> io::Result<FxHashMap<PathBuf, String>> {
  let directory = root
    .parent()
    .ok_or_else(|| io::Error::other("crate root has no directory"))?;
  let mut tree = SourceTree {
    package,
    generated,
    active: FxHashSet::default(),
    sources: Vec::new(),
  };
  tree.collect(root, &[], directory, directory)?;
  let sources = tree.sources;
  let mut imports = Imports::default();
  for source in &sources {
    imports.module.clone_from(&source.module);
    imports.visit_file(&source.parsed);
  }
  let mut manual = ManualAdapters {
    imports: imports.names,
    ..Default::default()
  };
  for source in &sources {
    manual.module.clone_from(&source.module);
    manual.visit_file(&source.parsed);
  }
  sources
    .into_iter()
    .map(|source| {
      let text = instrument_file(
        &source.parsed,
        &source.text,
        crate_path,
        &[],
        &manual.names,
        &source.module,
      )
      .map_err(io::Error::other)?;
      Ok((source.path, text))
    })
    .collect()
}

struct SourceTree<'a> {
  package: &'a Path,
  generated: Option<&'a Path>,
  active: FxHashSet<PathBuf>,
  sources: Vec<Source>,
}

impl SourceTree<'_> {
  fn collect(
    &mut self,
    path: &Path,
    module: &[String],
    directory: &Path,
    default_directory: &Path,
  ) -> io::Result<()> {
    let path = path.canonicalize()?;
    if !path.starts_with(self.package)
      || self
        .generated
        .is_some_and(|generated| path.starts_with(generated))
      || !self.active.insert(path.clone())
    {
      return Ok(());
    }
    let text = fs::read_to_string(&path)?;
    let parsed = syn::parse_file(&text).map_err(|error| {
      io::Error::other(format!(
        "allocative instrumentation of {}: {error}",
        path.display()
      ))
    })?;
    let mut modules = Modules {
      module: module.to_vec(),
      directory: directory.to_path_buf(),
      default_directory: default_directory.to_path_buf(),
      children: Vec::new(),
    };
    modules.visit_file(&parsed);
    self.sources.push(Source {
      path: path.clone(),
      module: module.to_vec(),
      text,
      parsed,
    });
    for (child, module, directory, default_directory) in modules.children {
      self.collect(&child, &module, &directory, &default_directory)?;
    }
    self.active.remove(&path);
    Ok(())
  }
}

struct Modules {
  module: Vec<String>,
  directory: PathBuf,
  default_directory: PathBuf,
  children: Vec<(PathBuf, Vec<String>, PathBuf, PathBuf)>,
}

impl<'ast> Visit<'ast> for Modules {
  fn visit_item_mod(&mut self, item: &'ast ItemMod) {
    let explicit = item.attrs.iter().find_map(|attr| {
      if attr.path().is_ident("path")
        && let syn::Meta::NameValue(value) = &attr.meta
        && let syn::Expr::Lit(value) = &value.value
        && let syn::Lit::Str(value) = &value.lit
      {
        Some(self.directory.join(value.value()))
      } else {
        None
      }
    });
    let name = item.ident.to_string();
    self.module.push(name.clone());
    if item.content.is_some() {
      let directory = explicit.unwrap_or_else(|| self.default_directory.join(&name));
      let previous_directory = std::mem::replace(&mut self.directory, directory.clone());
      let previous_default = std::mem::replace(&mut self.default_directory, directory);
      syn::visit::visit_item_mod(self, item);
      self.directory = previous_directory;
      self.default_directory = previous_default;
    } else {
      let (path, relative) = if let Some(path) = explicit {
        (path, false)
      } else {
        let path = self.default_directory.join(format!("{name}.rs"));
        if path.is_file() {
          (path, true)
        } else {
          (self.default_directory.join(&name).join("mod.rs"), false)
        }
      };
      // Disabled modules may refer to files that do not exist on this platform.
      // Rustc remains responsible for cfg evaluation and missing-module errors.
      if path.is_file()
        && let Some(directory) = path.parent()
      {
        let default = if relative {
          directory.join(name)
        } else {
          directory.to_path_buf()
        };
        self.children.push((
          path.clone(),
          self.module.clone(),
          directory.to_path_buf(),
          default,
        ));
      }
    }
    self.module.pop();
  }
}
