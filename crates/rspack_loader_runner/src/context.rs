use std::sync::Arc;

use derive_more::Debug;
use rspack_cacheable::cacheable;
use rspack_error::Diagnostic;
use rspack_paths::{InternedPath, InternedPathSet, Utf8Path};
use rspack_sources::SourceMap;

use crate::{
  AdditionalData, Content, LoaderItem, LoaderRunnerPlugin, ParseMeta, ResourceData,
  loader::LoaderItemList,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
  Init,
  Pitching,
  ProcessResource,
  Normal,
  Finished,
}

impl State {
  pub(crate) fn transition(&mut self, next: State) {
    *self = match (*self, next) {
      (State::Init, State::Pitching) => State::Pitching,
      (State::Pitching, State::ProcessResource) => State::ProcessResource,
      (State::Pitching, State::Normal) => State::Normal, // if pitching loader modifies the content
      (State::ProcessResource, State::Normal) => State::Normal,
      (State::Normal, State::Finished) => State::Finished,
      _ => panic!("Unexpected loader runner state (current: {self:?}, next: {next:?})"),
    };
  }
}

#[cacheable]
#[derive(Clone, Debug, Default)]
pub struct LoaderDependencies {
  pub file: InternedPathSet,
  pub context: InternedPathSet,
  pub missing: InternedPathSet,
  pub build: InternedPathSet,
}

impl LoaderDependencies {
  pub fn is_empty(&self) -> bool {
    self.file.is_empty()
      && self.context.is_empty()
      && self.missing.is_empty()
      && self.build.is_empty()
  }
}

#[derive(Debug)]
pub struct LoaderContext<Context: Send> {
  pub hot: bool,
  pub resource_data: Arc<ResourceData>,
  #[debug(skip)]
  pub context: Context,
  pub parse_meta: ParseMeta,

  pub(crate) content: Option<Content>,
  pub(crate) source_map: Option<Box<SourceMap<'static>>>,
  pub(crate) additional_data: Option<AdditionalData>,

  pub cacheable: bool,
  /// Dependencies committed by resource processing and preceding loaders.
  pub(crate) dependencies: LoaderDependencies,
  /// Dependencies added by the current native loader. A dependency remains
  /// here even when it was already present in `dependencies`.
  pub(crate) added_dependencies: LoaderDependencies,
  /// Dependencies removed by the current native loader.
  pub(crate) removed_dependencies: LoaderDependencies,

  pub diagnostics: Vec<Diagnostic>,

  /// Loader States
  pub(crate) state: State,
  pub loader_index: i32,
  pub loader_items: Vec<LoaderItem<Context>>,
  #[debug(skip)]
  pub plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
}

impl<Context: Send> LoaderContext<Context> {
  pub fn dependencies(&self) -> &LoaderDependencies {
    &self.dependencies
  }

  pub fn file_dependencies(&self) -> &InternedPathSet {
    &self.dependencies.file
  }

  pub fn context_dependencies(&self) -> &InternedPathSet {
    &self.dependencies.context
  }

  pub fn missing_dependencies(&self) -> &InternedPathSet {
    &self.dependencies.missing
  }

  pub fn build_dependencies(&self) -> &InternedPathSet {
    &self.dependencies.build
  }

  #[doc(hidden)]
  pub fn added_dependencies(&self) -> &LoaderDependencies {
    &self.added_dependencies
  }

  #[doc(hidden)]
  pub fn removed_dependencies(&self) -> &LoaderDependencies {
    &self.removed_dependencies
  }

  #[doc(hidden)]
  pub fn reset_dependency_changes(&mut self) {
    self.added_dependencies = Default::default();
    self.removed_dependencies = Default::default();
  }

  #[doc(hidden)]
  pub fn merge_dependency_changes(&mut self) {
    macro_rules! merge_dependencies {
      ($field:ident) => {{
        for dependency in self.removed_dependencies.$field.drain() {
          self.dependencies.$field.remove(&dependency);
        }
        self
          .dependencies
          .$field
          .extend(self.added_dependencies.$field.drain());
      }};
    }

    merge_dependencies!(file);
    merge_dependencies!(context);
    merge_dependencies!(missing);
    merge_dependencies!(build);
  }

  #[doc(hidden)]
  pub fn replace_dependencies(&mut self, dependencies: LoaderDependencies) {
    self.dependencies = dependencies;
    self.reset_dependency_changes();
  }

  #[doc(hidden)]
  pub fn add_dependencies(&mut self, dependencies: &LoaderDependencies) {
    for dependency in &dependencies.file {
      self.add_file_dependency(dependency.clone());
    }
    for dependency in &dependencies.context {
      self.add_context_dependency(dependency.clone());
    }
    for dependency in &dependencies.missing {
      self.add_missing_dependency(dependency.clone());
    }
    for dependency in &dependencies.build {
      self.add_build_dependency(dependency.clone());
    }
  }

  pub fn add_file_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.removed_dependencies.file.remove(&dependency);
    self.added_dependencies.file.insert(dependency);
  }

  pub fn add_context_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.removed_dependencies.context.remove(&dependency);
    self.added_dependencies.context.insert(dependency);
  }

  pub fn add_missing_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.removed_dependencies.missing.remove(&dependency);
    self.added_dependencies.missing.insert(dependency);
  }

  pub fn add_build_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.removed_dependencies.build.remove(&dependency);
    self.added_dependencies.build.insert(dependency);
  }

  pub fn remove_file_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.added_dependencies.file.remove(&dependency);
    if self.dependencies.file.contains(&dependency) {
      self.removed_dependencies.file.insert(dependency);
    }
  }

  pub fn remove_context_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.added_dependencies.context.remove(&dependency);
    if self.dependencies.context.contains(&dependency) {
      self.removed_dependencies.context.insert(dependency);
    }
  }

  pub fn remove_missing_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.added_dependencies.missing.remove(&dependency);
    if self.dependencies.missing.contains(&dependency) {
      self.removed_dependencies.missing.insert(dependency);
    }
  }

  pub fn remove_build_dependency(&mut self, dependency: impl Into<InternedPath>) {
    let dependency = dependency.into();
    self.added_dependencies.build.remove(&dependency);
    if self.dependencies.build.contains(&dependency) {
      self.removed_dependencies.build.insert(dependency);
    }
  }

  pub fn clear_dependencies(&mut self) {
    self
      .removed_dependencies
      .file
      .extend(self.dependencies.file.iter().cloned());
    self
      .removed_dependencies
      .context
      .extend(self.dependencies.context.iter().cloned());
    self
      .removed_dependencies
      .missing
      .extend(self.dependencies.missing.iter().cloned());
    self.added_dependencies.file.clear();
    self.added_dependencies.context.clear();
    self.added_dependencies.missing.clear();
  }

  pub fn remaining_request(&self) -> LoaderItemList<'_, Context> {
    if self.loader_index >= self.loader_items.len() as i32 - 1 {
      return Default::default();
    }
    LoaderItemList(&self.loader_items[self.loader_index as usize + 1..])
  }

  pub fn previous_request(&self) -> LoaderItemList<'_, Context> {
    LoaderItemList(&self.loader_items[..self.loader_index as usize])
  }

  #[inline]
  pub fn current_loader(&self) -> &LoaderItem<Context> {
    &self.loader_items[self.loader_index as usize]
  }

  /// Emit a diagnostic, it can be a `warning` or `error`.
  pub fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic)
  }

  /// The resource part of the request, including query and fragment.
  /// E.g. /abc/resource.js?query=1#some-fragment
  pub fn resource(&self) -> &str {
    self.resource_data.resource()
  }

  /// The resource part of the request.
  /// E.g. /abc/resource.js
  pub fn resource_path(&self) -> Option<&Utf8Path> {
    self.resource_data.path()
  }

  /// The query of the request
  /// E.g. query=1
  pub fn resource_query(&self) -> Option<&str> {
    self.resource_data.query()
  }

  pub fn content(&self) -> Option<&Content> {
    self.content.as_ref()
  }

  pub fn source_map(&self) -> Option<&SourceMap<'static>> {
    self.source_map.as_deref()
  }

  pub fn additional_data(&self) -> Option<&AdditionalData> {
    self.additional_data.as_ref()
  }

  pub fn take_content(&mut self) -> Option<Content> {
    self.content.take()
  }

  pub fn take_source_map(&mut self) -> Option<SourceMap<'static>> {
    self.source_map.take().map(|source_map| *source_map)
  }

  pub fn take_additional_data(&mut self) -> Option<AdditionalData> {
    self.additional_data.take()
  }

  pub fn take_all(
    &mut self,
  ) -> (
    Option<Content>,
    Option<SourceMap<'static>>,
    Option<AdditionalData>,
  ) {
    (
      self.content.take(),
      self.take_source_map(),
      self.additional_data.take(),
    )
  }

  pub fn finish_with(&mut self, patch: impl Into<LoaderPatch>) {
    self.__finish_with(patch);
    self.current_loader().set_finish_called();
  }

  pub fn finish_with_empty(&mut self) {
    self.content = None;
    self.source_map = None;
    self.additional_data = None;
    self.current_loader().set_finish_called();
  }

  #[inline]
  pub fn state(&self) -> State {
    self.state
  }

  #[doc(hidden)]
  pub fn __finish_with(&mut self, patch: impl Into<LoaderPatch>) {
    let patch = patch.into();
    self.content = patch.content;
    self.source_map = patch.source_map.map(Box::new);
    self.additional_data = patch.additional_data;
  }
}

pub struct LoaderPatch {
  pub(crate) content: Option<Content>,
  pub(crate) source_map: Option<SourceMap<'static>>,
  pub(crate) additional_data: Option<AdditionalData>,
}

impl<T> From<T> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(content: T) -> Self {
    Self {
      content: Some(content.into()),
      source_map: None,
      additional_data: None,
    }
  }
}

impl<T> From<(T, SourceMap<'static>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, SourceMap<'static>)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: Some(value.1),
      additional_data: None,
    }
  }
}

impl<T> From<(T, Option<SourceMap<'static>>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, Option<SourceMap<'static>>)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: value.1,
      additional_data: None,
    }
  }
}

impl<T> From<(T, SourceMap<'static>, AdditionalData)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, SourceMap<'static>, AdditionalData)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: Some(value.1),
      additional_data: Some(value.2),
    }
  }
}

impl<T> From<(T, Option<SourceMap<'static>>, Option<AdditionalData>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, Option<SourceMap<'static>>, Option<AdditionalData>)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: value.1,
      additional_data: value.2,
    }
  }
}

impl<T> From<Option<T>> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(content: Option<T>) -> Self {
    Self {
      content: content.map(|c| c.into()),
      source_map: None,
      additional_data: None,
    }
  }
}

impl<T> From<(Option<T>, SourceMap<'static>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, SourceMap<'static>)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: Some(value.1),
      additional_data: None,
    }
  }
}

impl<T> From<(Option<T>, Option<SourceMap<'static>>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, Option<SourceMap<'static>>)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: value.1,
      additional_data: None,
    }
  }
}

impl<T> From<(Option<T>, SourceMap<'static>, AdditionalData)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, SourceMap<'static>, AdditionalData)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: Some(value.1),
      additional_data: Some(value.2),
    }
  }
}

impl<T>
  From<(
    Option<T>,
    Option<SourceMap<'static>>,
    Option<AdditionalData>,
  )> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(
    value: (
      Option<T>,
      Option<SourceMap<'static>>,
      Option<AdditionalData>,
    ),
  ) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: value.1,
      additional_data: value.2,
    }
  }
}
