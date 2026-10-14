use std::{cmp::Reverse, hash::BuildHasherDefault, sync::Arc};

use camino::{Utf8Path, Utf8PathBuf};
use cow_utils::CowUtils;
use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxHasher;
use serde::Deserialize;

use crate::path::PathUtil;

pub type CompilerOptionsPathsMap = IndexMap<String, Vec<String>, BuildHasherDefault<FxHasher>>;
pub type FileDependencies = IndexSet<Utf8PathBuf, BuildHasherDefault<FxHasher>>;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum ExtendsField {
  Single(String),
  Multiple(Vec<String>),
}

const TEMPLATE_VARIABLE: &str = "${configDir}";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
  /// Whether this is the caller tsconfig.
  /// Used for final template variable substitution when all configs are extended and merged.
  #[serde(skip)]
  root: bool,

  /// Path to `tsconfig.json`. Contains the `tsconfig.json` filename.
  #[serde(skip)]
  pub(crate) path: Utf8PathBuf,

  #[serde(skip)]
  pub(crate) file_dependencies: FileDependencies,

  #[serde(default)]
  pub extends: Option<ExtendsField>,

  #[serde(default)]
  pub compiler_options: CompilerOptions,

  /// Bubbled up project references with a reference to their tsconfig.
  #[serde(default)]
  pub references: Vec<ProjectReference>,

  /// Flattened transitive project references in resolution order.
  #[serde(skip)]
  pub(crate) flattened_references: Vec<Arc<Self>>,
}

/// Compiler Options
///
/// <https://www.typescriptlang.org/tsconfig#compilerOptions>
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
  base_url: Option<Utf8PathBuf>,

  /// Path aliases
  paths: Option<CompilerOptionsPathsMap>,

  /// The actual base for where path aliases are resolved from.
  #[serde(skip)]
  paths_base: Utf8PathBuf,
}

/// Project Reference
///
/// <https://www.typescriptlang.org/docs/handbook/project-references.html>
#[derive(Debug, Deserialize)]
pub struct ProjectReference {
  /// The path property of each reference can point to a directory containing a tsconfig.json file,
  /// or to the config file itself (which may have any name).
  pub path: Utf8PathBuf,

  /// Reference to the resolved tsconfig
  #[serde(skip)]
  pub tsconfig: Option<Arc<TsConfig>>,
}

/// tsc's `PathIsRelative`: `.`, `..`, or a `./`, `../` (also `.\`, `..\`)
/// prefix. `paths` never applies to these; an absolute specifier is not relative.
pub(crate) fn is_relative_specifier(specifier: &str) -> bool {
  matches!(
    specifier.as_bytes(),
    [b'.'] | [b'.', b'.'] | [b'.', b'/' | b'\\', ..] | [b'.', b'.', b'/' | b'\\', ..]
  )
}

impl TsConfig {
  #[cfg_attr(feature="enable_instrument", tracing::instrument(level=tracing::Level::DEBUG, skip_all, fields(path = path.as_str())))]
  pub fn parse(root: bool, path: &Utf8Path, json: &mut str) -> Result<Self, serde_json::Error> {
    _ = json_strip_comments::strip(json);
    if json.trim().is_empty() {
      let mut tsconfig: Self = serde_json::from_str("{}")?;
      tsconfig.root = root;
      tsconfig.path = path.to_path_buf();
      tsconfig.file_dependencies.insert(path.to_path_buf());
      return Ok(tsconfig);
    }
    let mut tsconfig: Self = serde_json::from_str(json)?;
    tsconfig.root = root;
    tsconfig.path = path.to_path_buf();
    tsconfig.file_dependencies.insert(path.to_path_buf());
    let directory = tsconfig.directory().to_path_buf();
    if let Some(base_url) = &tsconfig.compiler_options.base_url {
      // keep the `${configDir}` template variable in the baseUrl
      if !base_url.starts_with(TEMPLATE_VARIABLE) {
        tsconfig.compiler_options.base_url = Some(directory.normalize_with(base_url));
      }
    }
    if tsconfig.compiler_options.paths.is_some() {
      tsconfig.compiler_options.paths_base = tsconfig
        .compiler_options
        .base_url
        .as_ref()
        .map_or(directory, Clone::clone);
    }
    Ok(tsconfig)
  }

  pub fn build(mut self) -> Self {
    if self.root {
      let dir = self.directory().to_path_buf();
      // Substitute template variable in `tsconfig.compilerOptions.paths`
      if let Some(paths) = &mut self.compiler_options.paths {
        for paths in paths.values_mut() {
          for path in paths {
            Self::substitute_template_variable(&dir, path);
          }
        }
      }

      let mut p = self.compiler_options.paths_base.as_str().to_string();
      Self::substitute_template_variable(&dir, &mut p);
      self.compiler_options.paths_base = p.into();

      if let Some(base_url) = self.compiler_options.base_url.as_mut() {
        let mut p = base_url.as_str().to_string();
        Self::substitute_template_variable(&dir, &mut p);
        *base_url = p.into();
      }
    }
    self
  }

  /// Directory to `tsconfig.json`
  ///
  /// # Panics
  ///
  /// * When the `tsconfig.json` path is misconfigured.
  pub fn directory(&self) -> &Utf8Path {
    debug_assert!(self.path.file_name().is_some());
    self
      .path
      .parent()
      .expect("tsconfig path must have a parent")
  }

  pub fn extend_tsconfig(&mut self, other_config: &Self) {
    let compiler_options = &mut self.compiler_options;
    if compiler_options.paths.is_none() {
      compiler_options.paths_base = compiler_options.base_url.as_ref().map_or_else(
        || other_config.compiler_options.paths_base.clone(),
        Clone::clone,
      );
      compiler_options
        .paths
        .clone_from(&other_config.compiler_options.paths);
    }
    if compiler_options.base_url.is_none() {
      compiler_options
        .base_url
        .clone_from(&other_config.compiler_options.base_url);
    }
    self
      .file_dependencies
      .extend(other_config.file_dependencies.iter().cloned());
  }

  /// `paths` act like aliases across the configs claiming `path`: the most
  /// specific config with a `paths` key matching `specifier` supplies the
  /// mapped candidates, so a config without such a key falls through to the
  /// next one. Its targets are not padded with other configs' mappings, so a
  /// missing target never resolves to a same-named file elsewhere. The
  /// `baseUrl` candidate likewise comes from the most specific config with a
  /// `baseUrl`, after the `paths` candidates.
  pub fn resolve(&self, path: &Utf8Path, specifier: &str) -> Vec<Utf8PathBuf> {
    if specifier.starts_with(['/', '.']) {
      return vec![];
    }
    let configs = self.configs_by_specificity(path);
    let paths = configs
      .iter()
      .find_map(|config| config.paths_candidates(specifier))
      .unwrap_or_default();
    let base_url = configs
      .iter()
      .find_map(|config| config.base_url_candidate(specifier));
    paths.into_iter().chain(base_url).collect()
  }

  /// Configs whose claimed directory is a prefix of `path`, longest first.
  /// References with neither `paths` nor `baseUrl` are skipped. On a tie, a
  /// config with `paths` goes first, then a reference before the root
  /// config, then references in declaration order, as on main. The root
  /// config always comes last when it does not claim `path`.
  fn configs_by_specificity(&self, path: &Utf8Path) -> Vec<&Self> {
    let root_claims_path = path.starts_with(self.directory());
    let mut configs: Vec<(&Utf8Path, bool, &Self)> = self
      .flattened_references
      .iter()
      .filter(|reference| reference.has_path_alias())
      .map(|reference| (reference.claimed_directory(), true, reference.as_ref()))
      .filter(|(directory, _, _)| path.starts_with(directory))
      .chain(root_claims_path.then(|| (self.directory(), false, self)))
      .collect();
    configs.sort_by_key(|(directory, is_reference, config)| {
      Reverse((
        directory.as_str().len(),
        config.compiler_options.paths.is_some(),
        *is_reference,
      ))
    });
    let mut configs: Vec<&Self> = configs.into_iter().map(|(_, _, config)| config).collect();
    if !root_claims_path {
      configs.push(self);
    }
    configs
  }

  /// Whether `resolve_path_alias` can produce any candidate.
  fn has_path_alias(&self) -> bool {
    self.compiler_options.paths.is_some() || self.compiler_options.base_url.is_some()
  }

  /// The directory a reference applies to: its config directory when it
  /// declares `paths`, like `enhanced-resolve`, otherwise its `baseUrl`, as
  /// on main.
  fn claimed_directory(&self) -> &Utf8Path {
    match &self.compiler_options {
      CompilerOptions {
        paths: None,
        base_url: Some(base_url),
        ..
      } => base_url,
      _ => self.directory(),
    }
  }

  // Copied from parcel
  // <https://github.com/parcel-bundler/parcel/blob/b6224fd519f95e68d8b93ba90376fd94c8b76e69/packages/utils/node-resolver-rs/src/tsconfig.rs#L93>
  pub fn resolve_path_alias(&self, specifier: &str) -> Vec<Utf8PathBuf> {
    if specifier.starts_with(['/', '.']) {
      return vec![];
    }
    self
      .paths_candidates(specifier)
      .unwrap_or_default()
      .into_iter()
      .chain(self.base_url_candidate(specifier))
      .collect()
  }

  /// The `paths` targets for `specifier`, or `None` when no `paths` key
  /// matches it.
  fn paths_candidates(&self, specifier: &str) -> Option<Vec<Utf8PathBuf>> {
    let paths_map = self.compiler_options.paths.as_ref()?;

    let paths = paths_map.get(specifier).cloned().or_else(|| {
      let mut longest_prefix_length = 0;
      let mut longest_suffix_length = 0;
      let mut best_key: Option<&String> = None;

      for key in paths_map.keys() {
        if let Some((prefix, suffix)) = key.split_once('*') {
          if (best_key.is_none() || prefix.len() > longest_prefix_length)
            && specifier.starts_with(prefix)
            && specifier.ends_with(suffix)
          {
            longest_prefix_length = prefix.len();
            longest_suffix_length = suffix.len();
            best_key.replace(key);
          }
        }
      }

      best_key.and_then(|key| paths_map.get(key)).map(|paths| {
        paths
          .iter()
          .map(|path| {
            path
              .cow_replace(
                '*',
                &specifier[longest_prefix_length..specifier.len() - longest_suffix_length],
              )
              .into_owned()
          })
          .collect::<Vec<_>>()
      })
    })?;

    Some(
      paths
        .into_iter()
        .map(|p| self.compiler_options.paths_base.normalize_with(p))
        .collect(),
    )
  }

  fn base_url_candidate(&self, specifier: &str) -> Option<Utf8PathBuf> {
    self
      .compiler_options
      .base_url
      .as_ref()
      .map(|base_url| base_url.normalize_with(specifier))
  }

  /// Template variable `${configDir}` for substitution of config files directory path
  ///
  /// NOTE: All tests cases are just a head replacement of `${configDir}`, so we are constrained as such.
  ///
  /// See <https://github.com/microsoft/TypeScript/pull/58042>
  fn substitute_template_variable(directory: &Utf8Path, path: &mut String) {
    if let Some(stripped_path) = path.strip_prefix(TEMPLATE_VARIABLE) {
      if let Some(unleashed_path) = stripped_path.strip_prefix("/") {
        *path = directory.join(unleashed_path).into_string();
      } else {
        *path = directory.join(stripped_path).into_string();
      }
    }
  }
}
