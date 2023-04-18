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
  /// It will be `None` at pitching stage.
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

  pub(crate) loader_index: usize,
  pub(crate) loader_items: LoaderItemList<'c, C>,

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

  pub(crate) fn current_loader(&self) -> &LoaderItem<C> {
    &self.loader_items[self.loader_index]
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
    if let Some(processed_resource) = plugin.process_resource(resource_data).await? {
      loader_context.content = Some(processed_resource);
    }
  }

  if loader_context.content.is_none() {
    let result = tokio::fs::read(&resource_data.resource_path).await?;
    loader_context.content = Some(Content::from(result));
  }

  loader_context.loader_index = loader_context.loader_items.len() - 1;
  iterate_normal_loaders(loader_context).await
}

async fn create_loader_context<'c, C: 'c>(
  loader_items: &'c [LoaderItem<C>],
  resource_data: &'c ResourceData,
  context: C,
  is_composed: bool,
) -> Result<LoaderContext<'c, C>> {
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
async fn iterate_normal_loaders<C>(loader_context: &mut LoaderContext<'_, C>) -> Result<()> {
  let current_loader_item = loader_context.current_loader();

  if current_loader_item.normal_executed() {
    if loader_context.loader_index == 0 {
      return Ok(());
    }
    loader_context.loader_index -= 1;
    return iterate_normal_loaders(loader_context).await;
  }

  let loader = current_loader_item.loader.clone();
  current_loader_item.set_normal_executed();
  loader.run(loader_context).await?;

  iterate_normal_loaders(loader_context).await
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

  let current_loader_item = loader_context.current_loader();

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
    iterate_normal_loaders(loader_context).await?;
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
    .iter()
    .map(|i| i.clone().into())
    .collect::<Vec<LoaderItem<C>>>();

  let mut loader_context =
    create_loader_context(&loaders[..], resource_data, context, false).await?;

  assert!(loader_context.content.is_none());
  iterate_pitching_loaders(&mut loader_context, resource_data, plugins).await?;

  Ok(())
}

#[cfg(test)]
#[allow(unused)]
mod test {
  use std::{cell::RefCell, sync::Arc};

  use rspack_error::Result;
  use rspack_identifier::{Identifiable, Identifier};

  use super::{run_loaders, Loader, LoaderContext, ResourceData};
  use crate::{
    content::Content,
    loader::test::{Composed, Custom, Custom2},
    plugin::LoaderRunnerPlugin,
    DisplayWithSuffix,
  };

  struct TestContentPlugin {}

  #[async_trait::async_trait]
  impl LoaderRunnerPlugin for TestContentPlugin {
    fn name(&self) -> &'static str {
      "test-content"
    }

    async fn process_resource(&self, _resource_data: &ResourceData) -> Result<Option<Content>> {
      Ok(Some(Content::Buffer(vec![])))
    }
  }

  #[tokio::test]
  async fn should_have_the_correct_requests() {
    thread_local! {
      static IDENTS: RefCell<Vec<String>> = RefCell::default();
    }

    struct Pitching {}

    impl Identifiable for Pitching {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for Pitching {
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| {
          i.borrow_mut()
            .push(loader_context.remaining_request().to_string());
          i.borrow_mut()
            .push(loader_context.previous_request().to_string());
          i.borrow_mut()
            .push(loader_context.current_request().to_string());
          i.borrow_mut().push(
            loader_context
              .current_request()
              .display_with_suffix(loader_context.resource),
          );
        });
        Ok(())
      }
    }
    let c1 = Arc::new(Custom {}) as Arc<dyn Loader<()>>;
    let i1 = c1.identifier();
    let c2 = Arc::new(Custom2 {}) as Arc<dyn Loader<()>>;
    let i2 = c2.identifier();
    let c3 = Arc::new(Composed {}) as Arc<dyn Loader<()>>;
    let i3 = c3.identifier();
    let p1 = Arc::new(Pitching {}) as Arc<dyn Loader<()>>;
    let i0 = p1.identifier();

    let rs = ResourceData {
      resource: "/rspack/main.js?abc=123#efg".to_owned(),
      resource_description: None,
      resource_fragment: None,
      resource_query: None,
      resource_path: Default::default(),
    };

    run_loaders(
      &[c1, p1, c2, c3],
      &rs,
      &[Box::new(TestContentPlugin {})],
      (),
    )
    .await
    .unwrap();

    IDENTS.with(|i| {
      let i = i.borrow();
      // remaining request
      let expected = "".to_owned() + &**i2 + "!" + &**i3;
      assert_eq!(i[0], expected);
      // previous request
      let expected = "".to_owned() + &**i1;
      assert_eq!(i[1], expected);
      // current request
      let expected = "".to_owned() + &**i0 + "!" + &**i2 + "!" + &**i3;
      assert_eq!(i[2], expected);
      let expected = expected + "!" + &*rs.resource;
      assert_eq!(i[3], expected);
    });
  }

  #[tokio::test]
  async fn should_have_the_right_execution_order() {
    thread_local! {
      static IDENTS: RefCell<Vec<String>> = RefCell::default();
    }

    struct Pitching {}

    impl Identifiable for Pitching {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader1".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for Pitching {
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch1".to_string()));
        Ok(())
      }
    }

    struct Pitching2 {}

    impl Identifiable for Pitching2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitching-loader2".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for Pitching2 {
      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch2".to_string()));
        Ok(())
      }
    }

    struct Normal {}

    impl Identifiable for Normal {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader1".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for Normal {
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal1".to_string()));
        Ok(())
      }
    }

    struct Normal2 {}

    impl Identifiable for Normal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/normal-loader2".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for Normal2 {
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("normal2".to_string()));
        Ok(())
      }
    }

    struct PitchNormalBase {}

    impl Identifiable for PitchNormalBase {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-base-loader".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for PitchNormalBase {
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-base-pitch".to_string()));
        Ok(())
      }
    }

    struct PitchNormal {}

    impl Identifiable for PitchNormal {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-loader".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for PitchNormal {
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-pitch".to_string()));
        loader_context.content = Some(Content::Buffer(vec![]));
        Ok(())
      }
    }

    struct PitchNormal2 {}

    impl Identifiable for PitchNormal2 {
      fn identifier(&self) -> Identifier {
        "/rspack/pitch-normal-2-loader".into()
      }
    }

    #[async_trait::async_trait(?Send)]
    impl Loader<()> for PitchNormal2 {
      async fn run(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-normal-2".to_string()));
        Ok(())
      }

      async fn pitch(&self, loader_context: &mut LoaderContext<'_, ()>) -> Result<()> {
        IDENTS.with(|i| i.borrow_mut().push("pitch-normal-pitch-2".to_string()));
        loader_context.content = Some(Content::Buffer(vec![]));
        Ok(())
      }
    }

    let c1 = Arc::new(Normal {}) as Arc<dyn Loader<()>>;
    let c2 = Arc::new(Normal2 {}) as Arc<dyn Loader<()>>;
    let p1 = Arc::new(Pitching {}) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(Pitching2 {}) as Arc<dyn Loader<()>>;

    let rs = ResourceData {
      resource: "/rspack/main.js?abc=123#efg".to_owned(),
      resource_description: None,
      resource_fragment: None,
      resource_query: None,
      resource_path: Default::default(),
    };

    run_loaders::<()>(
      &[p1, p2, c1, c2],
      &rs,
      &[Box::new(TestContentPlugin {})],
      (),
    )
    .await
    .unwrap();
    IDENTS.with(|i| assert_eq!(*i.borrow(), &["pitch1", "pitch2", "normal2", "normal1"]));
    IDENTS.with(|i| i.borrow_mut().clear());

    let p1 = Arc::new(PitchNormalBase {}) as Arc<dyn Loader<()>>;
    let p2 = Arc::new(PitchNormal {}) as Arc<dyn Loader<()>>;
    let p3 = Arc::new(PitchNormal2 {}) as Arc<dyn Loader<()>>;

    run_loaders::<()>(&[p1, p2, p3], &rs, &[Box::new(TestContentPlugin {})], ())
      .await
      .unwrap();
    IDENTS.with(|i| {
      // should not execute p3, as p2 pitched successfully.
      assert!(!i.borrow().contains(&"pitch-normal-normal-2".to_string()));
      assert!(!i.borrow().contains(&"pitch-normal-pitch-2".to_string()));
      // should skip normal stage of p2.
      assert!(!i.borrow().contains(&"pitch-normal-normal".to_string()));
      // should still run the normal stage of pitch normal base.
      assert_eq!(i.borrow()[0], "pitch-normal-base-pitch".to_string());
      assert_eq!(i.borrow()[2], "pitch-normal-base-normal".to_string());
      // p2 pitched successfully.
      assert_eq!(i.borrow()[1], "pitch-normal-pitch".to_string());
    });
  }
}
