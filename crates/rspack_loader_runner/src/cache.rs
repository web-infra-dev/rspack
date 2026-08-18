use std::any::Any;

use rspack_cacheable::cacheable;
use rspack_error::Result;

use crate::LoaderContext;

#[cacheable]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LoaderRunnerOptions {
  pub cache: bool,
  /// Loader name, stable options and loader version/file hash.
  pub cache_key: String,
}

pub struct LoaderCacheState(Box<dyn Any + Send + Sync>);

impl std::fmt::Debug for LoaderCacheState {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    formatter.write_str("LoaderCacheState(..)")
  }
}

impl LoaderCacheState {
  pub fn new(value: impl Any + Send + Sync) -> Self {
    Self(Box::new(value))
  }

  pub fn downcast<T: Any + Send + Sync>(self) -> std::result::Result<Box<T>, Self> {
    match self.0.downcast::<T>() {
      Ok(value) => Ok(value),
      Err(value) => Err(Self(value)),
    }
  }
}

#[derive(Debug)]
pub enum LoaderCacheAction {
  Disabled,
  Hit,
  Miss(LoaderCacheState),
}

impl LoaderCacheAction {
  pub fn is_hit(&self) -> bool {
    matches!(self, Self::Hit)
  }
}

pub(crate) async fn before_normal_loader<Context: Send>(
  context: &mut LoaderContext<Context>,
) -> Result<LoaderCacheAction> {
  if !context.current_loader().cache() {
    return Ok(LoaderCacheAction::Disabled);
  }
  let Some(plugin) = context.plugin.clone() else {
    return Ok(LoaderCacheAction::Disabled);
  };
  plugin.before_normal_loader(context).await
}

pub(crate) async fn after_normal_loader<Context: Send>(
  context: &mut LoaderContext<Context>,
  action: LoaderCacheAction,
) -> Result<()> {
  let LoaderCacheAction::Miss(state) = action else {
    return Ok(());
  };
  if let Some(plugin) = context.plugin.clone() {
    plugin.after_normal_loader(context, state).await?;
  }
  Ok(())
}
