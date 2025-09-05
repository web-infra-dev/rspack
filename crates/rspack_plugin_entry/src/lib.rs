use std::sync::LazyLock;

use rspack_core::{
  BoxDependency, Compilation, CompilationParams, CompilerCompilation, CompilerMake, Context,
  DependencyType, EntryDependency, EntryOptions, Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};

type LazyDependency = LazyLock<BoxDependency, Box<dyn FnOnce() -> BoxDependency + Send>>;

#[plugin]
#[derive(Debug)]
pub struct EntryPlugin {
  // Need "cache" the dependency to tell incremental that this entry dependency is not changed
  // so it can be reused and skip the module make
  dependency: LazyDependency,
  options: EntryOptions,
}

impl EntryPlugin {
  pub fn new(context: Context, entry_request: String, options: EntryOptions) -> Self {
    let layer = options.layer.clone();
    let name = options.name.is_none();
    let dependency: LazyDependency = LazyLock::new(Box::new(move || {
      Box::new(EntryDependency::new(entry_request, context, layer, name))
    }));

    Self::new_inner(dependency, options)
  }
}

#[plugin_hook(CompilerCompilation for EntryPlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(DependencyType::Entry, params.normal_module_factory.clone());
  Ok(())
}

#[plugin_hook(CompilerMake for EntryPlugin)]
async fn make(&self, compilation: &mut Compilation) -> Result<()> {
  let this = &self.inner;
  compilation
    .add_entry(this.dependency.clone(), this.options.clone())
    .await?;
  Ok(())
}

impl Plugin for EntryPlugin {
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx.compiler_hooks.make.tap(make::new(self));
    Ok(())
  }
}
