use std::{path::PathBuf, sync::Arc};

use derive_more::Debug;
use rspack_error::Diagnostic;
use rspack_paths::Utf8Path;
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

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

#[derive(Debug)]
pub struct LoaderContext<Context: Send> {
  pub hot: bool,
  pub resource_data: Arc<ResourceData>,
  #[debug(skip)]
  pub context: Context,
  pub parse_meta: ParseMeta,

  pub(crate) content: Option<Content>,
  pub(crate) source_map: Option<SourceMap>,
  pub(crate) additional_data: Option<AdditionalData>,

  pub cacheable: bool,
  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,

  pub diagnostics: Vec<Diagnostic>,

  /// Loader States
  pub(crate) state: State,
  pub loader_index: i32,
  pub loader_items: Vec<LoaderItem<Context>>,
  #[debug(skip)]
  pub plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
}

impl<Context: Send> LoaderContext<Context> {
  pub fn remaining_request(&self) -> LoaderItemList<'_, Context> {
    if self.loader_index >= self.loader_items.len() as i32 - 1 {
      return Default::default();
    }
    LoaderItemList(&self.loader_items[self.loader_index as usize + 1..])
  }

  pub fn current_request(&self) -> LoaderItemList<'_, Context> {
    LoaderItemList(&self.loader_items[self.loader_index as usize..])
  }

  pub fn previous_request(&self) -> LoaderItemList<'_, Context> {
    LoaderItemList(&self.loader_items[..self.loader_index as usize])
  }

  pub fn request(&self) -> LoaderItemList<'_, Context> {
    LoaderItemList(&self.loader_items[..])
  }

  #[inline]
  pub fn current_loader(&self) -> &LoaderItem<Context> {
    &self.loader_items[self.loader_index as usize]
  }

  /// Emit a diagnostic, it can be a `warning` or `error`.
  pub fn emit_diagnostic(&mut self, diagnostic: Diagnostic) {
    self.diagnostics.push(diagnostic)
  }

  pub fn resource_data(&self) -> &ResourceData {
    &self.resource_data
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

  /// The fragment of the request
  /// E.g. some-fragment
  pub fn resource_fragment(&self) -> Option<&str> {
    self.resource_data.fragment()
  }

  pub fn content(&self) -> Option<&Content> {
    self.content.as_ref()
  }

  pub fn source_map(&self) -> Option<&SourceMap> {
    self.source_map.as_ref()
  }

  pub fn additional_data(&self) -> Option<&AdditionalData> {
    self.additional_data.as_ref()
  }

  pub fn take_content(&mut self) -> Option<Content> {
    self.content.take()
  }

  pub fn take_source_map(&mut self) -> Option<SourceMap> {
    self.source_map.take()
  }

  pub fn take_additional_data(&mut self) -> Option<AdditionalData> {
    self.additional_data.take()
  }

  pub fn take_all(&mut self) -> (Option<Content>, Option<SourceMap>, Option<AdditionalData>) {
    (
      self.content.take(),
      self.source_map.take(),
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
    self.source_map = patch.source_map;
    self.additional_data = patch.additional_data;
  }
}

pub struct LoaderPatch {
  pub(crate) content: Option<Content>,
  pub(crate) source_map: Option<SourceMap>,
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

impl<T> From<(T, SourceMap)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, SourceMap)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: Some(value.1),
      additional_data: None,
    }
  }
}

impl<T> From<(T, Option<SourceMap>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, Option<SourceMap>)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: value.1,
      additional_data: None,
    }
  }
}

impl<T> From<(T, SourceMap, AdditionalData)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, SourceMap, AdditionalData)) -> Self {
    Self {
      content: Some(value.0.into()),
      source_map: Some(value.1),
      additional_data: Some(value.2),
    }
  }
}

impl<T> From<(T, Option<SourceMap>, Option<AdditionalData>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (T, Option<SourceMap>, Option<AdditionalData>)) -> Self {
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

impl<T> From<(Option<T>, SourceMap)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, SourceMap)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: Some(value.1),
      additional_data: None,
    }
  }
}

impl<T> From<(Option<T>, Option<SourceMap>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, Option<SourceMap>)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: value.1,
      additional_data: None,
    }
  }
}

impl<T> From<(Option<T>, SourceMap, AdditionalData)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, SourceMap, AdditionalData)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: Some(value.1),
      additional_data: Some(value.2),
    }
  }
}

impl<T> From<(Option<T>, Option<SourceMap>, Option<AdditionalData>)> for LoaderPatch
where
  T: Into<Content>,
{
  fn from(value: (Option<T>, Option<SourceMap>, Option<AdditionalData>)) -> Self {
    Self {
      content: value.0.map(|c| c.into()),
      source_map: value.1,
      additional_data: value.2,
    }
  }
}
