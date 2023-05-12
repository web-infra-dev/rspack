#![allow(clippy::comparison_chain)]
mod impl_plugin_for_css_plugin;
use std::cmp;
use std::hash::Hash;
use std::str::FromStr;

use anyhow::bail;
use bitflags::bitflags;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::{Chunk, ChunkGraph, Compilation, Module, ModuleGraph, PathData, SourceType};
use rspack_core::{Filename, ModuleIdentifier};
use rspack_identifier::IdentifierSet;

use crate::pxtorem::options::PxToRemOptions;

static ESCAPE_LOCAL_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\.]"#).expect("Invalid regex"));

#[derive(Debug)]
pub struct CssPlugin {
  config: CssConfig,
}

#[derive(Debug, Clone, Default)]
pub struct PostcssConfig {
  pub pxtorem: Option<PxToRemOptions>,
}

#[derive(Debug, Clone)]
pub struct ModulesConfig {
  pub locals_convention: LocalsConvention,
  pub local_ident_name: LocalIdentName,
  pub exports_only: bool,
}

#[derive(Debug, Clone)]
pub struct LocalIdentName(Filename);

impl LocalIdentName {
  pub fn render(&self, options: LocalIdentNameRenderOptions) -> String {
    let mut s = self.0.render(options.path_data, None);
    if let Some(local) = options.local {
      s = s.replace("[local]", local);
    }
    s = ESCAPE_LOCAL_IDENT_REGEX.replace_all(&s, "-").into_owned();
    s
  }
}

impl From<String> for LocalIdentName {
  fn from(value: String) -> Self {
    Self(Filename::from(value))
  }
}

pub struct LocalIdentNameRenderOptions<'a> {
  pub path_data: PathData<'a>,
  pub local: Option<&'a str>,
}

bitflags! {
  struct LocalsConventionFlags: u8 {
    const ASIS = 1 << 0;
    const CAMELCASE = 1 << 1;
    const DASHES = 1 << 2;
  }
}

#[derive(Debug, Clone)]
pub struct LocalsConvention(LocalsConventionFlags);

impl LocalsConvention {
  pub fn as_is(&self) -> bool {
    self.0.contains(LocalsConventionFlags::ASIS)
  }

  pub fn camel_case(&self) -> bool {
    self.0.contains(LocalsConventionFlags::CAMELCASE)
  }

  pub fn dashes(&self) -> bool {
    self.0.contains(LocalsConventionFlags::DASHES)
  }
}

impl FromStr for LocalsConvention {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(match s {
      "asIs" => Self(LocalsConventionFlags::ASIS),
      "camelCase" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::CAMELCASE),
      "camelCaseOnly" => Self(LocalsConventionFlags::CAMELCASE),
      "dashes" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::DASHES),
      "dashesOnly" => Self(LocalsConventionFlags::DASHES),
      _ => bail!("css modules exportsLocalsConvention error"),
    })
  }
}

impl Default for LocalsConvention {
  fn default() -> Self {
    Self(LocalsConventionFlags::ASIS)
  }
}

#[derive(Debug, Clone)]
pub struct CssConfig {
  pub targets: Vec<String>,
  pub postcss: PostcssConfig,
  pub modules: ModulesConfig,
}

impl CssPlugin {
  pub fn new(config: CssConfig) -> Self {
    Self { config }
  }

  pub(crate) fn get_ordered_chunk_css_modules<'module>(
    chunk: &Chunk,
    chunk_graph: &'module ChunkGraph,
    module_graph: &'module ModuleGraph,
    compilation: &Compilation,
  ) -> Vec<ModuleIdentifier> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L368
    let mut css_modules = chunk_graph
      .get_chunk_modules_iterable_by_source_type(&chunk.ukey, SourceType::Css, module_graph)
      .collect::<Vec<_>>();
    css_modules.sort_unstable_by_key(|module| module.identifier());

    let css_modules: Vec<ModuleIdentifier> =
      Self::get_modules_in_order(chunk, css_modules, compilation);

    css_modules
  }

  pub(crate) fn get_modules_in_order(
    chunk: &Chunk,
    modules: Vec<&dyn Module>,
    compilation: &Compilation,
  ) -> Vec<ModuleIdentifier> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L269
    if modules.is_empty() {
      return vec![];
    };

    let modules_list = modules.into_iter().map(|m| m.identifier()).collect_vec();

    // Get ordered list of modules per chunk group
    // Lists are in reverse order to allow to use Array.pop()
    let mut modules_by_chunk_group = chunk
      .groups
      .iter()
      .filter_map(|ukey| compilation.chunk_group_by_ukey.get(ukey))
      .map(|chunk_group| {
        let sorted_modules = modules_list
          .clone()
          .into_iter()
          .filter_map(|module_id| {
            let order = chunk_group.module_post_order_index(&module_id);
            order.map(|order| (module_id, order))
          })
          .sorted_by(|a, b| {
            if b.1 > a.1 {
              std::cmp::Ordering::Less
            } else if b.1 < a.1 {
              std::cmp::Ordering::Greater
            } else {
              std::cmp::Ordering::Equal
            }
          })
          .map(|item| item.0)
          .collect_vec();

        SortedModules {
          set: sorted_modules.clone().into_iter().collect(),
          list: sorted_modules,
        }
      })
      .collect::<Vec<_>>();

    if modules_by_chunk_group.len() == 1 {
      return modules_by_chunk_group[0].list.clone();
    };

    modules_by_chunk_group.sort_unstable_by(compare_module_lists);

    let mut final_modules: Vec<ModuleIdentifier> = vec![];

    loop {
      let mut failed_modules: IdentifierSet = Default::default();
      let list = modules_by_chunk_group[0].list.clone();
      if list.is_empty() {
        // done, everything empty
        break;
      }
      let mut selected_module = *list.last().expect("TODO:");
      let mut has_failed = None;
      'outer: loop {
        for SortedModules { set, list } in &modules_by_chunk_group {
          if list.is_empty() {
            continue;
          }
          let last_module = *list.last().expect("TODO:");
          if last_module != selected_module {
            continue;
          }
          if !set.contains(&selected_module) {
            continue;
          }
          failed_modules.insert(selected_module);
          if failed_modules.contains(&last_module) {
            // There is a conflict, try other alternatives
            has_failed = Some(last_module);
            continue;
          }
          selected_module = last_module;
          has_failed = None;
          continue 'outer;
        }
        break;
      }
      if let Some(has_failed) = has_failed {
        // There is a not resolve-able conflict with the selectedModule
        // TODO(hyf0): we should emit a warning here
        tracing::warn!("Conflicting order between");
        // 		if (compilation) {
        // 			// TODO print better warning
        // 			compilation.warnings.push(
        // 				new Error(
        // 					`chunk ${
        // 						chunk.name || chunk.id
        // 					}\nConflicting order between ${hasFailed.readableIdentifier(
        // 						compilation.requestShortener
        // 					)} and ${selectedModule.readableIdentifier(
        // 						compilation.requestShortener
        // 					)}`
        // 				)
        // 			);
        // 		}

        selected_module = has_failed;
      }
      // Insert the selected module into the final modules list
      final_modules.push(selected_module);
      // Remove the selected module from all lists
      for SortedModules { set, list } in &mut modules_by_chunk_group {
        let last_module = list.last();
        if last_module.map_or(false, |last_module| last_module == &selected_module) {
          list.pop();
          set.remove(&selected_module);
        } else if has_failed.is_some() && set.contains(&selected_module) {
          let idx = list.iter().position(|m| m == &selected_module);
          if let Some(idx) = idx {
            list.remove(idx);
          }
        }
      }

      modules_by_chunk_group.sort_unstable_by(compare_module_lists);
    }
    final_modules
  }
}

struct SortedModules {
  pub list: Vec<ModuleIdentifier>,
  pub set: IdentifierSet,
}

fn compare_module_lists(a: &SortedModules, b: &SortedModules) -> cmp::Ordering {
  let a = &a.list;
  let b = &b.list;
  if a.is_empty() {
    if b.is_empty() {
      cmp::Ordering::Equal
    } else {
      cmp::Ordering::Greater
    }
  } else if b.is_empty() {
    cmp::Ordering::Less
  } else {
    compare_modules_by_identifier(a.last().expect("TODO:"), b.last().expect("TODO:"))
  }
}

fn compare_modules_by_identifier(a_id: &str, b_id: &str) -> cmp::Ordering {
  if a_id < b_id {
    cmp::Ordering::Less
  } else if a_id > b_id {
    cmp::Ordering::Greater
  } else {
    cmp::Ordering::Equal
  }
}
