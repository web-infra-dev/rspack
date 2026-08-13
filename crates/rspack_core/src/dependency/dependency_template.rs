use std::fmt::Debug;

use dyn_clone::{DynClone, clone_trait_object};
use rspack_cacheable::cacheable_dyn;
use rspack_hash::RspackHasher;
use rspack_sources::{ReplaceSource, ReplacementEnforce};
use rspack_util::ext::AsAny;

use crate::{
  ChunkInitFragments, CodeGenerationData, Compilation, ConcatenationScope, DependencyRange,
  DependencyType, Module, ModuleCodeTemplate, ModuleInitFragments, RuntimeSpec,
};

pub struct TemplateContext<'a, 'b> {
  pub compilation: &'a Compilation,
  pub module: &'a dyn Module,
  pub init_fragments: &'a mut ModuleInitFragments<'b>,
  pub runtime: Option<&'a RuntimeSpec>,
  pub data: &'a mut CodeGenerationData,
  pub runtime_template: &'a mut ModuleCodeTemplate,
}

impl TemplateContext<'_, '_> {
  pub fn chunk_init_fragments(&mut self) -> &mut ChunkInitFragments {
    let data_fragments = self.data.get::<ChunkInitFragments>();
    if data_fragments.is_some() {
      self
        .data
        .get_mut::<ChunkInitFragments>()
        .expect("should have chunk_init_fragments")
    } else {
      self.data.insert(ChunkInitFragments::default());
      self
        .data
        .get_mut::<ChunkInitFragments>()
        .expect("should have chunk_init_fragments")
    }
  }
}

/// The only source mutation entry point exposed to dependency templates.
///
/// Source edits and faster-concatenation scope updates are recorded together,
/// so a template cannot mutate one while forgetting the other.
pub struct TemplateReplaceSource<'a> {
  source: &'a mut ReplaceSource,
  concatenation_scope: Option<&'a mut ConcatenationScope>,
}

#[derive(Clone, Copy)]
enum GeneratedCodeUsedNames {
  Scan,
  AlreadyTracked,
}

impl<'a> TemplateReplaceSource<'a> {
  pub fn new(
    source: &'a mut ReplaceSource,
    concatenation_scope: Option<&'a mut ConcatenationScope>,
  ) -> Self {
    Self {
      source,
      concatenation_scope,
    }
  }

  pub fn concatenation_scope(&mut self) -> Option<&mut ConcatenationScope> {
    self.concatenation_scope.as_deref_mut()
  }

  pub fn faster_concatenation_scope(&mut self) -> Option<&mut ConcatenationScope> {
    self
      .concatenation_scope
      .as_deref_mut()
      .filter(|scope| scope.is_faster_module_concatenation())
  }

  pub fn ensure_generated_top_level_symbol(&mut self, preferred_name: impl Into<String>) -> String {
    let preferred_name = preferred_name.into();
    self
      .ensure_generated_top_level_symbol_in_scope(&preferred_name)
      .unwrap_or(preferred_name)
  }

  pub fn ensure_generated_top_level_symbol_in_scope(
    &mut self,
    preferred_name: &str,
  ) -> Option<String> {
    self.faster_concatenation_scope().map(|scope| {
      scope
        .ensure_generated_top_level_symbol(preferred_name)
        .to_string()
    })
  }

  #[inline]
  fn record_edit(
    &mut self,
    start: u32,
    end: u32,
    content: &str,
    used_names: GeneratedCodeUsedNames,
  ) {
    let Some(scope) = self.concatenation_scope.as_deref_mut() else {
      return;
    };
    scope.record_source_edit(
      (start != end).then(|| DependencyRange::new(start, end)),
      matches!(used_names, GeneratedCodeUsedNames::Scan).then_some(content),
    );
  }

  pub fn replace(&mut self, start: u32, end: u32, content: String, name: Option<String>) {
    self.record_edit(start, end, &content, GeneratedCodeUsedNames::Scan);
    self.source.replace(start, end, content, name);
  }

  /// Replaces source with code whose used names have already been recorded in
  /// the concatenation scope.
  pub fn replace_with_tracked_used_names(
    &mut self,
    start: u32,
    end: u32,
    content: String,
    name: Option<String>,
  ) {
    self.record_edit(start, end, &content, GeneratedCodeUsedNames::AlreadyTracked);
    self.source.replace(start, end, content, name);
  }

  pub fn replace_static(
    &mut self,
    start: u32,
    end: u32,
    content: &'static str,
    name: Option<&'static str>,
  ) {
    self.record_edit(start, end, content, GeneratedCodeUsedNames::Scan);
    self.source.replace_static(start, end, content, name);
  }

  pub fn replace_static_with_enforce(
    &mut self,
    start: u32,
    end: u32,
    content: &'static str,
    name: Option<&'static str>,
    enforce: ReplacementEnforce,
  ) {
    self.record_edit(start, end, content, GeneratedCodeUsedNames::Scan);
    self
      .source
      .replace_static_with_enforce(start, end, content, name, enforce);
  }

  pub fn insert(&mut self, start: u32, content: String, name: Option<String>) {
    self.record_edit(start, start, &content, GeneratedCodeUsedNames::Scan);
    self.source.insert(start, content, name);
  }

  pub fn insert_static(&mut self, start: u32, content: &'static str, name: Option<&'static str>) {
    self.record_edit(start, start, content, GeneratedCodeUsedNames::Scan);
    self.source.insert_static(start, content, name);
  }

  /// Ignores identifiers in a range even when this template intentionally
  /// leaves the source edit to an overlapping dependency.
  pub fn ignore_original_scope_range(&mut self, range: DependencyRange) {
    if let Some(scope) = self.faster_concatenation_scope() {
      scope.remove_original_range(range);
    }
  }

  /// Expands an object shorthand into a key-value pair where the original
  /// shorthand identifier is no longer an identifier reference.
  pub fn insert_shorthand_value(
    &mut self,
    start: u32,
    shorthand_range: DependencyRange,
    content: String,
    name: Option<String>,
  ) {
    self.insert_shorthand_value_with_used_names(
      start,
      shorthand_range,
      content,
      name,
      GeneratedCodeUsedNames::Scan,
    );
  }

  /// Expands a shorthand using code whose used names have already been
  /// recorded in the concatenation scope.
  pub fn insert_shorthand_value_with_tracked_used_names(
    &mut self,
    start: u32,
    shorthand_range: DependencyRange,
    content: String,
    name: Option<String>,
  ) {
    self.insert_shorthand_value_with_used_names(
      start,
      shorthand_range,
      content,
      name,
      GeneratedCodeUsedNames::AlreadyTracked,
    );
  }

  fn insert_shorthand_value_with_used_names(
    &mut self,
    start: u32,
    shorthand_range: DependencyRange,
    content: String,
    name: Option<String>,
    used_names: GeneratedCodeUsedNames,
  ) {
    if let Some(scope) = self.concatenation_scope.as_deref_mut() {
      scope.record_source_edit(
        Some(shorthand_range),
        matches!(used_names, GeneratedCodeUsedNames::Scan).then_some(content.as_str()),
      );
    }
    self.source.insert(start, content, name);
  }

  /// Expands an object shorthand while preserving the original identifier as
  /// the value or binding that still needs concatenation-time renaming.
  pub fn insert_non_shorthand(
    &mut self,
    start: u32,
    original_range: DependencyRange,
    content: String,
    name: Option<String>,
  ) {
    if let Some(scope) = self.concatenation_scope.as_deref_mut() {
      scope.record_non_shorthand_source_edit(original_range, &content);
    }
    self.source.insert(start, content, name);
  }
}

clone_trait_object!(DependencyCodeGeneration);

// Align with https://github.com/webpack/webpack/blob/671ac29d462e75a10c3fdfc785a4c153e41e749e/lib/DependencyCodeGeneration.js
#[cacheable_dyn]
pub trait DependencyCodeGeneration: Debug + DynClone + Sync + Send + AsAny {
  fn update_hash(
    &self,
    _hasher: &mut RspackHasher,
    _compilation: &Compilation,
    _runtime: Option<&RuntimeSpec>,
  ) {
  }

  fn dependency_template(&self) -> Option<DependencyTemplateType> {
    None
  }
}

pub type BoxDependencyTemplate = Box<dyn DependencyCodeGeneration>;

pub trait AsDependencyCodeGeneration {
  fn as_dependency_code_generation(&self) -> Option<&dyn DependencyCodeGeneration> {
    None
  }
}

impl<T: DependencyCodeGeneration> AsDependencyCodeGeneration for T {
  fn as_dependency_code_generation(&self) -> Option<&dyn DependencyCodeGeneration> {
    Some(self)
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum DependencyTemplateType {
  Dependency(DependencyType),
  Custom(&'static str),
}

pub trait DependencyTemplate: Debug + Sync + Send {
  fn render(
    &self,
    dep: &dyn DependencyCodeGeneration,
    source: &mut TemplateReplaceSource,
    code_generatable_context: &mut TemplateContext,
  );
}
