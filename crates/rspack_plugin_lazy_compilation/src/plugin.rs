use std::{
  fmt::Debug,
  sync::{Arc, LazyLock},
};

use rspack_collections::IdentifierSet;
use rspack_core::{
  BoxModule, Compilation, CompilationId, CompilationParams, CompilerCompilation, CompilerId,
  CompilerMake, DependencyType, EntryDependency, LibIdentOptions, Module, ModuleExt, ModuleFactory,
  ModuleFactoryCreateData, ModuleIdentifier, NormalModuleCreateData, NormalModuleFactoryModule,
  Plugin,
};
use rspack_error::Result;
use rspack_hook::{plugin, plugin_hook};
use rspack_regex::RspackRegex;
use tokio::sync::{Mutex, RwLock};

use crate::{
  backend::Backend, factory::LazyCompilationDependencyFactory, module::LazyCompilationProxyModule,
  utils::calc_value_dependency_key,
};

static DEV_SERVER_CLIENT_RE: LazyLock<RspackRegex> = LazyLock::new(|| {
  RspackRegex::new(
    r#"(webpack|rspack)[/\\]hot[/\\]|(webpack|rspack)-dev-server[/\\]client|(webpack|rspack)-hot-middleware[/\\]client"#,
  )
  .expect("should compile regex")
});

#[derive(Debug, Hash, Clone)]
pub enum LazyCompilationTest<F: LazyCompilationTestCheck> {
  Regex(RspackRegex),
  Fn(F),
}

#[async_trait::async_trait]
pub trait LazyCompilationTestCheck: Send + Sync + Debug {
  async fn test(
    &self,
    compiler_id: CompilerId,
    compilation_id: CompilationId,
    module: &dyn Module,
  ) -> bool;
}

impl<F: LazyCompilationTestCheck> LazyCompilationTest<F> {
  async fn test(
    &self,
    compiler_id: CompilerId,
    compilation_id: CompilationId,
    module: &dyn Module,
  ) -> bool {
    match self {
      LazyCompilationTest::Regex(regex) => {
        regex.test(&module.name_for_condition().unwrap_or("".into()))
      }
      LazyCompilationTest::Fn(f) => f.test(compiler_id, compilation_id, module).await,
    }
  }
}

#[derive(Debug)]
#[plugin]
pub struct LazyCompilationPlugin<T: Backend, F: LazyCompilationTestCheck> {
  backend: Mutex<T>,
  entries: bool, // enable for entries
  imports: bool, // enable for imports
  test: Option<LazyCompilationTest<F>>,
  client: String,
  active_modules: RwLock<IdentifierSet>,
}

impl<T: Backend, F: LazyCompilationTestCheck> LazyCompilationPlugin<T, F> {
  pub fn new(
    backend: T,
    test: Option<LazyCompilationTest<F>>,
    entries: bool,
    imports: bool,
    client: String,
  ) -> Self {
    Self::new_inner(
      Mutex::new(backend),
      entries,
      imports,
      test,
      client,
      Default::default(),
    )
  }

  async fn check_test(
    &self,
    compiler_id: CompilerId,
    compilation_id: CompilationId,
    module: &BoxModule,
  ) -> bool {
    if let Some(test) = &self.inner.test {
      test
        .test(compiler_id, compilation_id, module.as_ref())
        .await
    } else {
      true
    }
  }
}

#[plugin_hook(CompilerCompilation for LazyCompilationPlugin<T: Backend, F: LazyCompilationTestCheck>)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  params: &mut CompilationParams,
) -> Result<()> {
  compilation.set_dependency_factory(
    DependencyType::LazyImport,
    Arc::new(LazyCompilationDependencyFactory::new(
      params.normal_module_factory.clone(),
    )) as Arc<dyn ModuleFactory>,
  );

  compilation
    .value_cache_versions
    .insert(calc_value_dependency_key("client"), self.client.clone());

  Ok(())
}

#[plugin_hook(NormalModuleFactoryModule for LazyCompilationPlugin<T: Backend, F: LazyCompilationTestCheck>)]
async fn normal_module_factory_module(
  &self,
  module_factory_create_data: &mut ModuleFactoryCreateData,
  create_data: &NormalModuleCreateData,
  module: &mut BoxModule,
) -> Result<()> {
  let dep_type = module_factory_create_data.dependencies[0].dependency_type();

  if matches!(dep_type, DependencyType::LazyImport) {
    return Ok(());
  };

  let is_imports = matches!(
    dep_type,
    DependencyType::DynamicImport
      | DependencyType::DynamicImportEager
      | DependencyType::ContextElement(rspack_core::ContextTypePrefix::Import)
  );
  let is_entries = matches!(dep_type, DependencyType::Entry);

  #[allow(clippy::if_same_then_else)]
  if matches!(
    dep_type,
    DependencyType::ModuleHotAccept
      | DependencyType::ModuleHotDecline
      | DependencyType::ImportMetaHotAccept
      | DependencyType::ImportMetaHotDecline
  ) {
    // TODO: we cannot access module graph at this stage
    // if hmr point to a module that is already been dyn imported
    // eg: import('./foo'); module.hot.accept('./foo')
    // however we cannot access module graph at this time, so we cannot
    // detect this case easily
    return Ok(());
  } else if !is_entries && !is_imports {
    return Ok(());
  }

  if is_entries {
    if !self.entries {
      return Ok(());
    }

    // ignore global entry
    let entry: Option<&EntryDependency> = module_factory_create_data.dependencies[0].downcast_ref();
    let Some(entry) = entry else {
      return Ok(());
    };

    if entry.is_global() {
      return Ok(());
    }
  }

  if !self.imports && is_imports {
    return Ok(());
  }

  if DEV_SERVER_CLIENT_RE.test(create_data.resource_resolve_data.resource())
    || !self
      .check_test(
        module_factory_create_data.compiler_id,
        module_factory_create_data.compilation_id,
        module,
      )
      .await
  {
    return Ok(());
  }

  let module_identifier: ModuleIdentifier =
    format!("lazy-compilation-proxy|{}", module.identifier()).into();
  let readable_identifier = format!(
    "lazy-compilation-proxy|{}",
    module_factory_create_data
      .context
      .shorten(&module.identifier())
  );
  let active = self
    .active_modules
    .read()
    .await
    .contains(&module_identifier);
  let lib_ident = module.lib_ident(LibIdentOptions {
    context: module_factory_create_data.options.context.as_str(),
  });

  *module = LazyCompilationProxyModule::new(
    module_identifier,
    readable_identifier,
    lib_ident.map(|ident| ident.into_owned()),
    module_factory_create_data,
    create_data.resource_resolve_data.resource().to_owned(),
    active,
    is_entries,
    self.client.clone(),
  )
  .boxed();

  Ok(())
}

#[plugin_hook(CompilerMake for LazyCompilationPlugin<T: Backend, F: LazyCompilationTestCheck>)]
async fn compiler_make(&self, compilation: &mut Compilation) -> Result<()> {
  let active_modules = self.backend.lock().await.current_active_modules().await?;

  // Proxy modules that current entries point to in this compilation.
  // A previously-active entry proxy whose entry has been removed (e.g. by a
  // dynamic entry function) must NOT be force-rebuilt here — its underlying
  // resource may no longer exist, and the orphan will be cleaned up later by
  // `BuildEntryAndClean` in `finish_module_graph`. The dep -> module lookup
  // works because incremental mode preserves the connection across builds;
  // entries whose deps haven't been factorized yet (first build, freshly
  // added entry) simply won't appear in this set, which is fine — we only
  // use it to *prove* stale-ness, never to drop ids we're unsure about.
  let current_entry_proxy_ids: IdentifierSet = {
    let mg = compilation.build_module_graph_artifact.get_module_graph();
    compilation
      .entries
      .values()
      .flat_map(|e| e.all_dependencies())
      .filter_map(|dep_id| mg.module_identifier_by_dependency_id(dep_id).copied())
      .collect()
  };

  let module_graph = compilation
    .build_module_graph_artifact
    .get_module_graph_mut();
  // Start from the full backend snapshot. Ids whose proxies haven't been
  // factorized yet (first build, new entry, non-incremental) must stay so
  // that `normal_module_factory_module`'s `active` check sees them later.
  // Only drop ids that we positively identify as stale entry proxies.
  let mut next_active_modules: IdentifierSet = active_modules.iter().copied().collect();
  for module_id in &active_modules {
    let Some(active_module) = module_graph.module_by_identifier_mut(module_id) else {
      continue;
    };
    let Some(active_module) = active_module.downcast_mut::<LazyCompilationProxyModule>() else {
      continue;
    };

    if active_module.is_entry() && !current_entry_proxy_ids.contains(module_id) {
      next_active_modules.remove(module_id);
      continue;
    }

    active_module.invalid();
  }

  *self.active_modules.write().await = next_active_modules;

  Ok(())
}

#[async_trait::async_trait]
impl<T: Backend + 'static, F: LazyCompilationTestCheck + 'static> Plugin
  for LazyCompilationPlugin<T, F>
{
  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));

    ctx
      .normal_module_factory_hooks
      .module
      .tap(normal_module_factory_module::new(self));

    ctx.compiler_hooks.make.tap(compiler_make::new(self));
    Ok(())
  }
}
