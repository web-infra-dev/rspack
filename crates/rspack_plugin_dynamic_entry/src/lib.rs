#![feature(let_chains)]

use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Debug;
use futures::future::BoxFuture;
use rspack_core::{
  ApplyContext, BoxDependency, Compilation, CompilationParams, CompilerCompilation, CompilerMake,
  CompilerOptions, Context, DependencyType, EntryDependency, EntryOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::{FxDashMap, FxHashMap};

pub struct EntryDynamicResult {
  pub import: Vec<String>,
  pub options: EntryOptions,
}

type EntryDynamic =
  Box<dyn for<'a> Fn() -> BoxFuture<'static, Result<Vec<EntryDynamicResult>>> + Sync + Send>;

pub struct DynamicEntryPluginOptions {
  pub context: Context,
  pub entry: EntryDynamic,
}

#[plugin]
#[derive(Debug)]
pub struct DynamicEntryPlugin {
  context: Context,
  #[debug(skip)]
  entry: EntryDynamic,
  // Need "cache" the dependency to tell incremental that this entry dependency is not changed
  // so it can be reused and skip the module make
  dependencies_map: FxDashMap<Arc<str>, FxHashMap<EntryOptions, BoxDependency>>,
}

impl DynamicEntryPlugin {
  pub fn new(options: DynamicEntryPluginOptions) -> Self {
    Self::new_inner(options.context, options.entry, Default::default())
  }
}

#[plugin_hook(CompilerCompilation for DynamicEntryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  Ok(())
}

#[plugin_hook(CompilerMake for DynamicEntryPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let entry_fn = &self.entry;
  let decs = entry_fn().await?;
  for EntryDynamicResult { import, options } in decs {
    for entry in import {
      let dependency: BoxDependency = if let Some(map) = self.dependencies_map.get(entry.as_str())
        && let Some(dependency) = map.get(&options)
      {
        dependency.clone()
      } else {
        let dependency: BoxDependency = Box::new(EntryDependency::new(
          entry.clone(),
          self.context.clone(),
          options.layer.clone(),
          false,
        ));
        if let Some(mut map) = self.dependencies_map.get_mut(entry.as_str()) {
          map.insert(options.clone(), dependency.clone());
        } else {
          let mut map = FxHashMap::default();
          map.insert(options.clone(), dependency.clone());
          self.dependencies_map.insert(entry.into(), map);
        }
        dependency
      };
      compilation.add_entry(dependency, options.clone()).await?;
    }
  }
  Ok(())
}

#[async_trait]
impl Plugin for DynamicEntryPlugin {
  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(compilation::new(self));
    ctx.context.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}
