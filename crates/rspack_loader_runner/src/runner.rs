use std::{
  collections::HashSet,
  fmt::Debug,
  path::{Path, PathBuf},
  sync::Arc,
};

use nodejs_resolver::DescriptionData;
use rspack_error::{Diagnostic, Result};
use rspack_sources::SourceMap;

use crate::{
  content::Content,
  loader::{Loader, LoaderItem, LoaderItemList},
  plugin::LoaderRunnerPlugin,
};

#[derive(Debug, Clone)]
pub struct ResourceData {
  /// Resource with absolute path, query and fragment
  pub resource: String,
  /// Absolute resource path only
  pub resource_path: PathBuf,
  /// Resource query with `?` prefix
  pub resource_query: Option<String>,
  /// Resource fragment with `#` prefix
  pub resource_fragment: Option<String>,
  pub resource_description: Option<Arc<DescriptionData>>,
}

#[derive(Debug)]
pub struct LoaderContext<'c, C> {
  /// Content of loader, represented by string or buffer
  /// Content should always be exist if at normal stage,
  /// However, it might be `None` at pitching stage.
  pub content: Option<Content>,

  /// The resource part of the request, including query and fragment.
  /// E.g. /abc/resource.js?query=1#some-fragment
  pub resource: &'c str,
  /// The resource part of the request.
  /// E.g. /abc/resource.js
  pub resource_path: &'c Path,
  /// The query of the request
  /// E.g. query=1
  pub resource_query: Option<&'c str>,
  /// The fragment of the request
  /// E.g. some-fragment
  pub resource_fragment: Option<&'c str>,

  pub context: C,
  pub source_map: Option<SourceMap>,
  pub additional_data: Option<String>,
  pub cacheable: bool,
  /// Is this a JS composed loader
  pub is_composed: bool,

  pub file_dependencies: HashSet<PathBuf>,
  pub context_dependencies: HashSet<PathBuf>,
  pub missing_dependencies: HashSet<PathBuf>,
  pub build_dependencies: HashSet<PathBuf>,

  pub loader_index: usize,
  pub loader_items: LoaderItemList<'c, C>,

  pub diagnostic: Vec<Diagnostic>,
}

impl<'c, C> LoaderContext<'c, C> {
  pub fn remaining_request(&self) -> LoaderItemList<'_, C> {
    if self.loader_index >= self.loader_items.len() - 1 {
      return Default::default();
    }
    LoaderItemList(&self.loader_items[self.loader_index + 1..])
  }

  pub fn current_request(&self) -> LoaderItemList<'_, C> {
    LoaderItemList(&self.loader_items[self.loader_index..])
  }

  pub fn previous_request(&self) -> LoaderItemList<'_, C> {
    LoaderItemList(&self.loader_items[..self.loader_index])
  }
}

/// Process resource
///
/// Plugins are loaded in order, if a plugin returns `Some(Content)`, then the returning content will be used as the result.
/// If plugins returned nothing, the runner will read via the `resource_path`.
async fn process_resource<C>(
  loader_context: &mut LoaderContext<'_, C>,
  resource_data: &ResourceData,
  plugins: &[Box<dyn LoaderRunnerPlugin>],
) -> Result<()> {
  for plugin in plugins {
    if let Some(processed_resource) = plugin.process_resource(&resource_data).await? {
      loader_context.content = Some(processed_resource);
      return Ok(());
    }
  }

  // let result = tokio::fs::read(&resource_data.resource_path).await?;
  // Ok(Content::from(result))
  loader_context.content = Some(Content::Buffer(vec![]));
  Ok(())
}

async fn create_loader_context<'c, C: 'c>(
  loader_items: &'c [LoaderItem<C>],
  resource_data: &'c ResourceData,
  plugins: &[Box<dyn LoaderRunnerPlugin>],
  context: C,
  is_composed: bool,
) -> Result<LoaderContext<'c, C>> {
  // let content = process_resource(resource_data, plugins).await?;

  // TODO: FileUriPlugin
  let mut file_dependencies: HashSet<PathBuf> = Default::default();
  file_dependencies.insert(resource_data.resource_path.clone());

  let loader_context = LoaderContext {
    cacheable: true,
    file_dependencies,
    context_dependencies: Default::default(),
    missing_dependencies: Default::default(),
    build_dependencies: Default::default(),
    content: None,
    resource: &resource_data.resource,
    resource_path: &resource_data.resource_path,
    resource_query: resource_data.resource_query.as_deref(),
    resource_fragment: resource_data.resource_fragment.as_deref(),
    context,
    is_composed,
    source_map: None,
    additional_data: None,
    loader_index: 0,
    loader_items: LoaderItemList(loader_items),
    diagnostic: vec![],
  };

  Ok(loader_context)
}

#[async_recursion::async_recursion(?Send)]
async fn iterate_normal_loaders<C>(
  loader_context: &mut LoaderContext<'_, C>,
  resource_data: &ResourceData,
) -> Result<()> {
  let current_loader_item = &loader_context.loader_items[loader_context.loader_index];

  if current_loader_item.normal_executed() {
    if loader_context.loader_index == 0 {
      return Ok(());
    }
    loader_context.loader_index -= 1;
    return iterate_normal_loaders(loader_context, resource_data).await;
  }

  let loader = current_loader_item.loader.clone();
  current_loader_item.set_normal_executed();
  loader.run(loader_context).await?;

  iterate_normal_loaders(loader_context, resource_data).await
}

#[async_recursion::async_recursion(?Send)]
async fn iterate_pitching_loaders<C>(
  loader_context: &mut LoaderContext<'_, C>,
  resource_data: &ResourceData,
  plugins: &[Box<dyn LoaderRunnerPlugin>],
) -> Result<()> {
  if loader_context.loader_index >= loader_context.loader_items.len() {
    return process_resource(loader_context, resource_data, plugins).await;
  }

  let current_loader_item = &loader_context.loader_items[loader_context.loader_index];

  if current_loader_item.pitch_executed() {
    loader_context.loader_index += 1;
    return iterate_pitching_loaders(loader_context, resource_data, plugins).await;
  }

  let loader = current_loader_item.loader.clone();
  current_loader_item.set_pitch_executed();
  loader.pitch(loader_context).await?;

  // If pitching loader modifies the content,
  // runner should skip the remaining pitching loaders
  // and redirect pipeline to the normal stage.
  if loader_context.content.is_some() {
    if loader_context.loader_index == 0 {
      return Ok(());
    }
    loader_context.loader_index -= 1;
    iterate_normal_loaders(loader_context, resource_data).await?;
  } else {
    iterate_pitching_loaders(loader_context, resource_data, plugins).await?;
  }

  Ok(())
}

pub async fn run_loaders<C: Debug>(
  loaders: &[Arc<dyn Loader<C>>],
  resource_data: &ResourceData,
  plugins: &[Box<dyn LoaderRunnerPlugin>],
  context: C,
) -> Result<()> {
  let loaders = loaders
    .into_iter()
    .map(|i| i.clone().into())
    .collect::<Vec<LoaderItem<C>>>();

  let mut loader_context =
    create_loader_context(&loaders[..], resource_data, plugins, context, false).await?;

  assert!(loader_context.content.is_none());
  iterate_pitching_loaders(&mut loader_context, resource_data, plugins).await?;

  Ok(())
}

#[cfg(test)]
mod test {
  use std::sync::Arc;

  use rspack_error::Result;
  use rspack_identifier::{Identifiable, Identifier};

  use super::{run_loaders, Loader, LoaderContext, ResourceData};
  use crate::loader::test::{Custom, Custom2};

  struct Pitching {}

  impl Identifiable for Pitching {
    fn identifier(&self) -> Identifier {
      "/rspack/pitching-loader".into()
    }
  }

  #[async_trait::async_trait(?Send)]
  impl Loader<()> for Pitching {
    async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
      dbg!(loader_context.remaining_request());
      dbg!(loader_context.previous_request());
      Ok(())
    }
  }

  #[tokio::test]
  async fn should_() {
    let c1 = Arc::new(Custom {}) as Arc<dyn Loader<()>>;
    let c2 = Arc::new(Custom2 {}) as Arc<dyn Loader<()>>;
    let p1 = Arc::new(Pitching {}) as Arc<dyn Loader<()>>;

    run_loaders(
      &[c1, p1, c2],
      &ResourceData {
        resource: Default::default(),
        resource_description: None,
        resource_fragment: None,
        resource_query: None,
        resource_path: Default::default(),
      },
      &[],
      (),
    )
    .await
    .unwrap();
  }
}
