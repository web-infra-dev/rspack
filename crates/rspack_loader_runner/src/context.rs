use std::{
  path::{Path, PathBuf},
  sync::Arc,
};

use derivative::Derivative;
use rspack_error::Diagnostic;
use rspack_sources::SourceMap;
use rustc_hash::FxHashSet as HashSet;

use crate::{
  loader::LoaderItemList, AdditionalData, Content, LoaderItem, LoaderRunnerPlugin, ResourceData,
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

#[derive(Derivative)]
#[derivative(Debug)]
pub struct LoaderContext<Context: 'static> {
  pub hot: bool,
  pub resource_data: Arc<ResourceData>,

  pub content: Option<Content>,
  #[derivative(Debug = "ignore")]
  pub context: Context,
  pub source_map: Option<SourceMap>,
  pub additional_data: AdditionalData,
  pub cacheable: bool,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,
  pub asset_filenames: HashSet<String>,

  pub diagnostics: Vec<Diagnostic>,

  /// Loader States
  pub(crate) state: State,
  pub loader_index: i32,
  pub loader_items: Vec<LoaderItem<Context>>,
  #[derivative(Debug = "ignore")]
  pub plugin: Option<Arc<dyn LoaderRunnerPlugin<Context = Context>>>,
}

impl<Context> LoaderContext<Context> {
  pub fn remaining_request(&self) -> LoaderItemList<Context> {
    if self.loader_index >= self.loader_items.len() as i32 - 1 {
      return Default::default();
    }
    LoaderItemList(&self.loader_items[self.loader_index as usize + 1..])
  }

  pub fn current_request(&self) -> LoaderItemList<Context> {
    LoaderItemList(&self.loader_items[self.loader_index as usize..])
  }

  pub fn previous_request(&self) -> LoaderItemList<Context> {
    LoaderItemList(&self.loader_items[..self.loader_index as usize])
  }

  pub fn request(&self) -> LoaderItemList<Context> {
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
    &self.resource_data.resource
  }

  /// The resource part of the request.
  /// E.g. /abc/resource.js
  pub fn resource_path(&self) -> Option<&Path> {
    self.resource_data.resource_path.as_deref()
  }

  /// The query of the request
  /// E.g. query=1
  pub fn resource_query(&self) -> Option<&str> {
    self.resource_data.resource_query.as_deref()
  }

  /// The fragment of the request
  /// E.g. some-fragment
  pub fn resource_fragment(&self) -> Option<&str> {
    self.resource_data.resource_fragment.as_deref()
  }

  #[inline]
  pub fn state(&self) -> State {
    self.state
  }
}
