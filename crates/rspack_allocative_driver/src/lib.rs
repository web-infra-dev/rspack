//! Source instrumentation shared by the allocative compiler driver and its regression tests.

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use rustc_hash::{FxHashMap, FxHashSet};
use syn::{
  ItemEnum, ItemImpl, ItemMod, ItemTrait, ItemUse, Type, UseTree, spanned::Spanned, visit::Visit,
};

mod source_tree;
pub use source_tree::instrument_source_tree;

/// Add derives to ordinary source enums, leaving macro token bodies untouched.
///
/// Insert attributes without pretty-printing the file, preserving comments and line numbers.
/// Rustc subsequently evaluates cfg attributes and expands the injected derive normally.
pub fn instrument(source: &str) -> syn::Result<String> {
  instrument_with_path(source, "::allocative")
}

/// Select the dependency path when a workspace crate reexports allocative through rspack_util.
pub fn instrument_with_path(source: &str, crate_path: &str) -> syn::Result<String> {
  instrument_with_traits(source, crate_path, &[])
}

/// Instrument the ownership interfaces of a workspace package as well as its enums.
pub fn instrument_crate(source: &str, crate_path: &str, crate_name: &str) -> syn::Result<String> {
  instrument_with_traits(source, crate_path, ownership_traits(crate_name))
}

fn ownership_traits(crate_name: &str) -> &'static [&'static str] {
  match crate_name {
    "rspack_core" => &[
      "Module",
      "ModuleFactory",
      "ParserAndGenerator",
      "Plugin",
      "Dependency",
      "DependencyConditionFn",
      "DependencyCodeGeneration",
      "DependencyTemplate",
      "InitFragment",
      "CodeGenerationDataItem",
      "Cache",
      "CacheValueObject",
    ],
    "rspack_sources" => &["Source"],
    "rspack_storage" => &["Storage"],
    "rspack_fs" => &["ReadableFileSystem", "WritableFileSystem"],
    "rspack_loader_runner" => &["Loader", "ParseMetaValue"],
    "rspack_binding_api" => &["VirtualFileStore"],
    "rspack_plugin_lazy_compilation" => &["Backend", "LazyCompilationTestCheck"],
    "rspack_plugin_javascript" => &["JavascriptParserPlugin"],
    _ => &[],
  }
}

fn instrument_with_traits(source: &str, crate_path: &str, traits: &[&str]) -> syn::Result<String> {
  let file = syn::parse_file(source)?;
  let mut imports = Imports::default();
  imports.visit_file(&file);
  let mut manual = ManualAdapters {
    imports: imports.names,
    ..Default::default()
  };
  manual.visit_file(&file);
  instrument_file(&file, source, crate_path, traits, &manual.names, &[])
}

fn instrument_file(
  file: &syn::File,
  source: &str,
  crate_path: &str,
  traits: &[&str],
  manual: &FxHashSet<String>,
  module: &[String],
) -> syn::Result<String> {
  let mut instrumentation = Instrumentation {
    manual,
    module: module.to_vec(),
    crate_path,
    traits,
    insertions: Vec::new(),
  };
  instrumentation.visit_file(file);
  let mut result = source.to_owned();
  instrumentation
    .insertions
    .sort_unstable_by_key(|(position, _)| *position);
  for (position, insertion) in instrumentation.insertions.into_iter().rev() {
    result.insert_str(position, &insertion);
  }
  Ok(result)
}

#[derive(Default)]
struct ManualAdapters {
  names: FxHashSet<String>,
  module: Vec<String>,
  imports: FxHashMap<String, Vec<String>>,
}

fn module_path(module: &[String], path: impl IntoIterator<Item = String>) -> Vec<String> {
  let mut module = module.to_vec();
  let mut path = path.into_iter().peekable();
  match path.peek().map(String::as_str) {
    Some("crate") => {
      module.clear();
      path.next();
    }
    Some("self") => {
      path.next();
    }
    _ => {
      while path.peek().is_some_and(|segment| segment == "super") {
        module.pop();
        path.next();
      }
    }
  }
  module.extend(path);
  module
}

#[derive(Default)]
struct Imports {
  names: FxHashMap<String, Vec<String>>,
  module: Vec<String>,
}

impl Imports {
  fn collect(&mut self, tree: &UseTree, prefix: &mut Vec<String>) {
    match tree {
      UseTree::Path(path) => {
        prefix.push(path.ident.to_string());
        self.collect(&path.tree, prefix);
        prefix.pop();
      }
      UseTree::Group(group) => {
        for tree in &group.items {
          self.collect(tree, prefix);
        }
      }
      UseTree::Name(name) => {
        let mut path = prefix.clone();
        let alias = if name.ident == "self" {
          prefix.last().cloned().unwrap_or_default()
        } else {
          path.push(name.ident.to_string());
          name.ident.to_string()
        };
        self.names.insert(
          module_path(&self.module, [alias]).join("::"),
          module_path(&self.module, path),
        );
      }
      UseTree::Rename(rename) => {
        let mut path = prefix.clone();
        if rename.ident != "self" {
          path.push(rename.ident.to_string());
        }
        self.names.insert(
          module_path(&self.module, [rename.rename.to_string()]).join("::"),
          module_path(&self.module, path),
        );
      }
      UseTree::Glob(_) => {}
    }
  }
}

impl<'ast> Visit<'ast> for Imports {
  fn visit_item_use(&mut self, item: &'ast ItemUse) {
    self.collect(&item.tree, &mut Vec::new());
  }

  fn visit_item_mod(&mut self, item: &'ast ItemMod) {
    self.module.push(item.ident.to_string());
    syn::visit::visit_item_mod(self, item);
    self.module.pop();
  }
}

impl<'ast> Visit<'ast> for ManualAdapters {
  fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
    if let Some((_, path, _)) = &item.trait_
      && path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Allocative")
      && let Type::Path(ty) = &*item.self_ty
    {
      let mut module = module_path(
        &self.module,
        ty.path
          .segments
          .iter()
          .map(|segment| segment.ident.to_string()),
      );
      let mut seen = FxHashSet::default();
      while seen.insert(module.clone()) {
        let replacement = (1..=module.len()).rev().find_map(|len| {
          self
            .imports
            .get(&module[..len].join("::"))
            .map(|target| (len, target.clone()))
        });
        let Some((len, mut target)) = replacement else {
          break;
        };
        target.extend_from_slice(&module[len..]);
        module = target;
      }
      self.names.insert(module.join("::"));
    }
    syn::visit::visit_item_impl(self, item);
  }

  fn visit_item_mod(&mut self, item: &'ast ItemMod) {
    self.module.push(item.ident.to_string());
    syn::visit::visit_item_mod(self, item);
    self.module.pop();
  }
}

struct Instrumentation<'a> {
  manual: &'a FxHashSet<String>,
  module: Vec<String>,
  crate_path: &'a str,
  traits: &'a [&'a str],
  insertions: Vec<(usize, String)>,
}

impl<'ast> Visit<'ast> for Instrumentation<'_> {
  fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
    let mut name = self.module.clone();
    name.push(item.ident.to_string());
    let explicit = item
      .attrs
      .iter()
      .any(|attr| contains_allocative(attr.meta.to_token_stream()));
    // Borrowed and const-generic enums retain their explicit typed adapters.
    // The derive itself selects typed traversal for nonliteral array lengths.
    if !explicit
      && !self.manual.contains(&name.join("::"))
      && item.generics.lifetimes().next().is_none()
      && item.generics.const_params().next().is_none()
    {
      self.insertions.push((
        item.span().byte_range().start,
        format!(
          "#[derive({0}::Allocative)] #[allocative(crate_path = \"{0}\")] ",
          self.crate_path
        ),
      ));
    }
    syn::visit::visit_item_enum(self, item);
  }

  fn visit_item_mod(&mut self, item: &'ast ItemMod) {
    self.module.push(item.ident.to_string());
    syn::visit::visit_item_mod(self, item);
    self.module.pop();
  }

  fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
    let selected = self.traits.contains(&item.ident.to_string().as_str());
    let explicit = item.supertraits.iter().any(|bound| {
      matches!(bound, syn::TypeParamBound::Trait(bound)
        if bound.path.segments.last().is_some_and(|segment| segment.ident == "Visit"))
    });
    if selected && !explicit {
      let (position, insertion) = if let Some(colon) = item.colon_token {
        // Keep attributes, comments and where clauses unchanged; insert before existing bounds.
        (
          colon.span.byte_range().end,
          format!(" {}::Visit +", self.crate_path),
        )
      } else {
        let end = item.generics.gt_token.map_or_else(
          || item.ident.span().byte_range().end,
          |token| token.span.byte_range().end,
        );
        (end, format!(": {}::Visit", self.crate_path))
      };
      self.insertions.push((position, insertion));
    }
    syn::visit::visit_item_trait(self, item);
  }
}

fn contains_allocative(tokens: TokenStream) -> bool {
  tokens.into_iter().any(|token| match token {
    TokenTree::Ident(ident) => ident == "Allocative",
    TokenTree::Group(group) => contains_allocative(group.stream()),
    _ => false,
  })
}
