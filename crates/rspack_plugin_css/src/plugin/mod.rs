#![allow(clippy::comparison_chain)]
mod impl_plugin_for_css_plugin;
use std::cmp::{self, Reverse};
use std::hash::Hash;
use std::str::FromStr;

use bitflags::bitflags;
use once_cell::sync::Lazy;
use regex::Regex;
use rspack_core::Filename;
use rspack_core::{Chunk, ChunkGraph, Compilation, Module, ModuleGraph, PathData, SourceType};
use rspack_error::error_bail;
use rspack_hook::plugin;
use rspack_identifier::IdentifierSet;

static ESCAPE_LOCAL_IDENT_REGEX: Lazy<Regex> =
  Lazy::new(|| Regex::new(r#"[<>:"/\\|?*\.]"#).expect("Invalid regex"));

#[plugin]
#[derive(Debug)]
pub struct CssPlugin {
  config: CssConfig,
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
  #[derive(Debug, Clone, Copy)]
  struct LocalsConventionFlags: u8 {
    const ASIS = 1 << 0;
    const CAMELCASE = 1 << 1;
    const DASHES = 1 << 2;
  }
}

#[derive(Debug, Clone, Copy)]
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
  type Err = rspack_error::Error;

  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    Ok(match s {
      "asIs" => Self(LocalsConventionFlags::ASIS),
      "camelCase" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::CAMELCASE),
      "camelCaseOnly" => Self(LocalsConventionFlags::CAMELCASE),
      "dashes" => Self(LocalsConventionFlags::ASIS | LocalsConventionFlags::DASHES),
      "dashesOnly" => Self(LocalsConventionFlags::DASHES),
      _ => error_bail!("css modules exportsLocalsConvention error"),
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
  pub modules: ModulesConfig,
  /// FIXME: Temp workaround, this option should be placed in `module.rules.*.parser`
  pub named_exports: Option<bool>,
}

impl CssPlugin {
  pub fn new(config: CssConfig) -> Self {
    Self::new_inner(config)
  }

  pub(crate) fn get_ordered_chunk_css_modules<'chunk_graph>(
    chunk: &Chunk,
    chunk_graph: &'chunk_graph ChunkGraph,
    module_graph: &'chunk_graph ModuleGraph,
    compilation: &Compilation,
  ) -> Vec<&'chunk_graph dyn Module> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L368
    let mut css_modules = chunk_graph
      .get_chunk_modules_iterable_by_source_type(&chunk.ukey, SourceType::Css, module_graph)
      .collect::<Vec<_>>();
    css_modules.sort_unstable_by_key(|module| module.identifier());

    let css_modules = Self::get_modules_in_order(chunk, css_modules, compilation);

    css_modules
  }

  pub(crate) fn get_modules_in_order<'module>(
    chunk: &Chunk,
    modules: Vec<&'module dyn Module>,
    compilation: &Compilation,
  ) -> Vec<&'module dyn Module> {
    // Align with https://github.com/webpack/webpack/blob/8241da7f1e75c5581ba535d127fa66aeb9eb2ac8/lib/css/CssModulesPlugin.js#L269
    if modules.is_empty() {
      return vec![];
    };

    let modules_list = modules.clone();

    // Get ordered list of modules per chunk group

    let mut modules_by_chunk_group = chunk
      .groups
      .iter()
      .map(|group| group.as_ref(&compilation.chunk_group_by_ukey))
      .map(|chunk_group| {
        let mut indexed_modules = modules_list
          .clone()
          .into_iter()
          .filter_map(|module| {
            // ->: import
            // For A -> B -> C, the pre order is A: 0, B: 1, C: 2.
            // The post order is A: 2, B: 1, C: 0.
            chunk_group
              .module_post_order_index(&module.identifier())
              .map(|index| (index, module))
          })
          .collect::<Vec<_>>();

        // After sort, we get a list [A, B, C].
        // Lists are in reverse order to allow to use `.pop()`
        let sorted_modules = {
          indexed_modules.sort_by_key(|item| Reverse(item.0));
          indexed_modules
            .into_iter()
            .map(|item| item.1)
            .collect::<Vec<_>>()
        };

        SortedModules {
          set: sorted_modules.iter().map(|m| m.identifier()).collect(),
          list: sorted_modules,
        }
      })
      .collect::<Vec<_>>();

    if modules_by_chunk_group.len() == 1 {
      let mut ret = modules_by_chunk_group
        .into_iter()
        .next()
        .expect("must have one")
        .list;
      ret.reverse();
      return ret;
    };

    modules_by_chunk_group.sort_unstable_by(compare_module_lists);

    let mut final_modules: Vec<&'module dyn Module> = vec![];

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
          if last_module == selected_module {
            continue;
          }
          if !set.contains(&selected_module.identifier()) {
            continue;
          }
          failed_modules.insert(selected_module.identifier());
          if failed_modules.contains(&last_module.identifier()) {
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
          set.remove(&selected_module.identifier());
        } else if has_failed.is_some() && set.contains(&selected_module.identifier()) {
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

#[derive(Debug)]
struct SortedModules<'module> {
  pub list: Vec<&'module dyn Module>,
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
    compare_modules_by_identifier(
      &a.last().expect("Must have a module").identifier(),
      &b.last().expect("Must have a module").identifier(),
    )
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
