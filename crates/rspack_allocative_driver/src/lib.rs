//! Source instrumentation shared by the allocative compiler driver and its regression tests.

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;
use rustc_hash::FxHashSet;
use syn::{ItemEnum, ItemImpl, ItemTrait, Type, spanned::Spanned, visit::Visit};

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
  let traits: &[&str] = match crate_name {
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
  };
  instrument_with_traits(source, crate_path, traits)
}

fn instrument_with_traits(source: &str, crate_path: &str, traits: &[&str]) -> syn::Result<String> {
  let file = syn::parse_file(source)?;
  let mut manual = ManualAdapters::default();
  manual.visit_file(&file);
  let mut instrumentation = Instrumentation {
    manual: &manual.names,
    crate_path,
    traits,
    insertions: Vec::new(),
  };
  instrumentation.visit_file(&file);
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
}

impl<'ast> Visit<'ast> for ManualAdapters {
  fn visit_item_impl(&mut self, item: &'ast ItemImpl) {
    if let Some((_, path, _)) = &item.trait_
      && path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Allocative")
      && let Type::Path(ty) = &*item.self_ty
      && let Some(segment) = ty.path.segments.last()
    {
      self.names.insert(segment.ident.to_string());
    }
    syn::visit::visit_item_impl(self, item);
  }
}

struct Instrumentation<'a> {
  manual: &'a FxHashSet<String>,
  crate_path: &'a str,
  traits: &'a [&'a str],
  insertions: Vec<(usize, String)>,
}

impl<'ast> Visit<'ast> for Instrumentation<'_> {
  fn visit_item_enum(&mut self, item: &'ast ItemEnum) {
    let explicit = item
      .attrs
      .iter()
      .any(|attr| contains_allocative(attr.meta.to_token_stream()));
    // Borrowed and const-array enums retain their explicit typed adapters. Ordinary enum
    // fields use reflection, which does not require third-party payloads to implement Allocative.
    let mut arrays = NonliteralArrays(false);
    arrays.visit_item_enum(item);
    if !explicit
      && !self.manual.contains(&item.ident.to_string())
      && item.generics.lifetimes().next().is_none()
      && item.generics.const_params().next().is_none()
      && !arrays.0
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

struct NonliteralArrays(bool);

impl<'ast> Visit<'ast> for NonliteralArrays {
  fn visit_type_array(&mut self, array: &'ast syn::TypeArray) {
    self.0 |= !matches!(array.len, syn::Expr::Lit(_));
    syn::visit::visit_type_array(self, array);
  }
}
