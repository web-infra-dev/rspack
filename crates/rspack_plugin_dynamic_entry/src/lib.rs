use std::sync::Arc;

use derive_more::Debug;
use futures::{future::BoxFuture, lock::Mutex};
use rspack_core::{
  BoxDependency, Compilation, CompilationParams, CompilerCompilation, CompilerMake, Context,
  DependencyId, DependencyType, EntryDependency, EntryOptions, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_util::fx_hash::FxHashMap;

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
  imported_dependencies: Mutex<FxHashMap<Arc<str>, FxHashMap<EntryOptions, DependencyId>>>,
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

  let mut imported_dependencies = self.imported_dependencies.lock().await;
  let mut next_imported_dependencies: FxHashMap<Arc<str>, FxHashMap<EntryOptions, DependencyId>> =
    Default::default();

  for EntryDynamicResult { import, options } in decs {
    for entry in import {
      let module_graph = compilation.get_module_graph();

      let entry_dependency: BoxDependency = if let Some(map) =
        imported_dependencies.get(entry.as_str())
        && let Some(dependency_id) = map.get(&options)
        && let Some(dependency) = module_graph.dependency_by_id(dependency_id)
      {
        next_imported_dependencies
          .entry(entry.into())
          .or_default()
          .insert(options.clone(), *dependency_id);
        dependency.clone()
      } else {
        let dependency: BoxDependency = Box::new(EntryDependency::new(
          entry.clone(),
          self.context.clone(),
          options.layer.clone(),
          false,
        ));
        next_imported_dependencies
          .entry(entry.into())
          .or_default()
          .insert(options.clone(), *dependency.id());
        dependency
      };
      compilation
        .add_entry(entry_dependency, options.clone())
        .await?;
    }
  }

  *imported_dependencies = next_imported_dependencies;

  Ok(())
}

impl Plugin for DynamicEntryPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}
