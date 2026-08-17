use std::{cmp::Ordering, path::PathBuf, sync::Arc};

use derive_more::Debug;
use rspack_error::Diagnostic;
use rspack_paths::Utf8Path;
use rspack_sources::SourceMap;
use rspack_util::ArcComputed;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  AdditionalData, Content, LoaderChain, LoaderItem, LoaderItemState, LoaderRunnerPlugin, Loaders,
  ParseMeta, ResourceData, loader::LoaderItemList,
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
  pub(crate) source_map: Option<Box<SourceMap<'static>>>,
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
  #[debug(skip)]
  pub loader_items: ArcComputed<Loaders<Context>, Vec<LoaderItem<Context>>>,
  pub loader_chains: ArcComputed<Loaders<Context>, Vec<LoaderChain>>,
  pub loader_item_states: Vec<LoaderItemState>,
  #[debug(skip)]
  pub plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
}

impl<Context: Send> LoaderContext<Context> {
  pub fn loader_items(&self) -> &[LoaderItem<Context>] {
    &self.loader_items
  }

  pub fn remaining_request(&self) -> LoaderItemList<'_, Context> {
    if self.loader_index >= self.loader_items().len() as i32 - 1 {
      return Default::default();
    }
    LoaderItemList(&self.loader_items()[self.loader_index as usize + 1..])
  }

  pub fn previous_request(&self) -> LoaderItemList<'_, Context> {
    LoaderItemList(&self.loader_items()[..self.loader_index as usize])
  }

  #[inline]
  pub fn current_loader(&self) -> &LoaderItem<Context> {
    &self.loader_items()[self.loader_index as usize]
  }

  pub fn loader_item_state(&self, index: usize) -> &LoaderItemState {
    &self.loader_item_states[index]
  }

  pub fn loader_item_state_mut(&mut self, index: usize) -> &mut LoaderItemState {
    &mut self.loader_item_states[index]
  }

  pub fn current_loader_state(&self) -> &LoaderItemState {
    self.loader_item_state(self.loader_index as usize)
  }

  pub fn current_loader_state_mut(&mut self) -> &mut LoaderItemState {
    self.loader_item_state_mut(self.loader_index as usize)
  }

  pub fn set_current_loader_pitch_executed(&mut self) {
    self.current_loader_state_mut().set_pitch_executed();
  }

  pub fn set_current_loader_normal_executed(&mut self) {
    self.current_loader_state_mut().set_normal_executed();
  }

  pub fn set_current_loader_finish_called(&mut self) {
    self.current_loader_state_mut().set_finish_called();
  }

  pub fn loader_chains(&self) -> &[LoaderChain] {
    &self.loader_chains
  }

  pub fn current_chain_index(&self) -> Option<usize> {
    let loader_index = usize::try_from(self.loader_index).ok()?;
    self
      .loader_chains
      .binary_search_by(|chain| {
        if chain.end() <= loader_index {
          Ordering::Less
        } else if chain.start() > loader_index {
          Ordering::Greater
        } else {
          Ordering::Equal
        }
      })
      .ok()
  }

  pub fn current_chain(&self) -> Option<&LoaderChain> {
    self
      .current_chain_index()
      .and_then(|index| self.loader_chains.get(index))
  }

  pub fn current_execution_chain(&self) -> Option<&LoaderChain> {
    self.current_chain()
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
    self.set_current_loader_finish_called();
  }

  pub fn finish_with_empty(&mut self) {
    self.content = None;
    self.source_map = None;
    self.additional_data = None;
    self.set_current_loader_finish_called();
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
