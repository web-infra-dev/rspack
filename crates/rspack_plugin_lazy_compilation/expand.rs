#![feature(prelude_import)]
#![feature(let_chains)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
pub mod backend {
  use rspack_core::ModuleIdentifier;
  use rspack_error::Result;
  pub struct ModuleInfo {
    pub active: bool,
    pub data: String,
    pub client: String,
  }
  pub trait Backend: std::fmt::Debug + Send + Sync {
    #[must_use]
    #[allow(clippy::type_complexity, clippy::type_repetition_in_bounds)]
    fn module<'life0, 'async_trait>(
      &'life0 mut self,
      original_module: ModuleIdentifier,
      path: String,
    ) -> ::core::pin::Pin<
      Box<
        dyn ::core::future::Future<Output = Result<ModuleInfo>>
          + ::core::marker::Send
          + 'async_trait,
      >,
    >
    where
      'life0: 'async_trait,
      Self: 'async_trait;
  }
}
mod dependency {
  use std::path::PathBuf;

  use rspack_core::{
    AsContextDependency, AsDependencyTemplate, Context, Dependency, DependencyCategory,
    DependencyId, DependencyType, ModuleDependency, NormalModuleCreateData, Resolve, ResourceData,
  };
  use rspack_error::Diagnostic;
  use rspack_identifier::Identifier;
  use rustc_hash::FxHashSet as HashSet;
  pub(crate) struct ProxyCreateData {
    pub resolve_options: Option<Box<Resolve>>,
    pub resource_resolve_data: ResourceData,
    pub context: Context,
    pub issuer: Option<Box<str>>,
    pub issuer_identifier: Option<Identifier>,
    pub file_dependencies: HashSet<PathBuf>,
    pub context_dependencies: HashSet<PathBuf>,
    pub missing_dependencies: HashSet<PathBuf>,
    pub diagnostics: Vec<Diagnostic>,
  }
  #[automatically_derived]
  impl ::core::fmt::Debug for ProxyCreateData {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      let names: &'static _ = &[
        "resolve_options",
        "resource_resolve_data",
        "context",
        "issuer",
        "issuer_identifier",
        "file_dependencies",
        "context_dependencies",
        "missing_dependencies",
        "diagnostics",
      ];
      let values: &[&dyn ::core::fmt::Debug] = &[
        &self.resolve_options,
        &self.resource_resolve_data,
        &self.context,
        &self.issuer,
        &self.issuer_identifier,
        &self.file_dependencies,
        &self.context_dependencies,
        &self.missing_dependencies,
        &&self.diagnostics,
      ];
      ::core::fmt::Formatter::debug_struct_fields_finish(f, "ProxyCreateData", names, values)
    }
  }
  #[automatically_derived]
  impl ::core::clone::Clone for ProxyCreateData {
    #[inline]
    fn clone(&self) -> ProxyCreateData {
      ProxyCreateData {
        resolve_options: ::core::clone::Clone::clone(&self.resolve_options),
        resource_resolve_data: ::core::clone::Clone::clone(&self.resource_resolve_data),
        context: ::core::clone::Clone::clone(&self.context),
        issuer: ::core::clone::Clone::clone(&self.issuer),
        issuer_identifier: ::core::clone::Clone::clone(&self.issuer_identifier),
        file_dependencies: ::core::clone::Clone::clone(&self.file_dependencies),
        context_dependencies: ::core::clone::Clone::clone(&self.context_dependencies),
        missing_dependencies: ::core::clone::Clone::clone(&self.missing_dependencies),
        diagnostics: ::core::clone::Clone::clone(&self.diagnostics),
      }
    }
  }
  impl ProxyCreateData {
    pub(crate) fn new(module_create_data: &NormalModuleCreateData) -> Self {
      Self {
        resolve_options: module_create_data.create_data.resolve_options.clone(),
        resource_resolve_data: module_create_data.resource_resolve_data.clone(),
        context: module_create_data.context.clone(),
        issuer: module_create_data.create_data.issuer.clone(),
        issuer_identifier: module_create_data.create_data.issuer_identifier,
        file_dependencies: module_create_data.create_data.file_dependencies.clone(),
        context_dependencies: module_create_data.create_data.context_dependencies.clone(),
        missing_dependencies: module_create_data.create_data.missing_dependencies.clone(),
        diagnostics: module_create_data.diagnostics.clone(),
      }
    }
  }
  pub(crate) struct LazyCompilationDependency {
    id: DependencyId,
    pub original_module_create_data: ProxyCreateData,
    request: String,
  }
  #[automatically_derived]
  impl ::core::fmt::Debug for LazyCompilationDependency {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      ::core::fmt::Formatter::debug_struct_field3_finish(
        f,
        "LazyCompilationDependency",
        "id",
        &self.id,
        "original_module_create_data",
        &self.original_module_create_data,
        "request",
        &&self.request,
      )
    }
  }
  #[automatically_derived]
  impl ::core::clone::Clone for LazyCompilationDependency {
    #[inline]
    fn clone(&self) -> LazyCompilationDependency {
      LazyCompilationDependency {
        id: ::core::clone::Clone::clone(&self.id),
        original_module_create_data: ::core::clone::Clone::clone(&self.original_module_create_data),
        request: ::core::clone::Clone::clone(&self.request),
      }
    }
  }
  impl LazyCompilationDependency {
    pub fn new(original_module_create_data: ProxyCreateData) -> Self {
      let request = {
        let res = ::alloc::fmt::format(format_args!(
          "{0}?lazy-compilation-proxy-dep",
          &original_module_create_data.resource_resolve_data.resource
        ));
        res
      };
      Self {
        id: DependencyId::new(),
        original_module_create_data,
        request,
      }
    }
  }
  impl ModuleDependency for LazyCompilationDependency {
    fn request(&self) -> &str {
      &self.request
    }
  }
  impl AsDependencyTemplate for LazyCompilationDependency {}
  impl AsContextDependency for LazyCompilationDependency {}
  impl Dependency for LazyCompilationDependency {
    fn dependency_debug_name(&self) -> &'static str {
      "lazy compilation dependency"
    }
    fn id(&self) -> &rspack_core::DependencyId {
      &self.id
    }
    fn category(&self) -> &DependencyCategory {
      &DependencyCategory::Esm
    }
    fn dependency_type(&self) -> &DependencyType {
      &DependencyType::LazyImport
    }
  }
}
mod factory {
  use std::sync::Arc;

  use rspack_core::{
    ModuleFactory, ModuleFactoryCreateData, ModuleFactoryResult, NormalModuleFactory,
  };
  use rspack_error::Result;

  use crate::dependency::LazyCompilationDependency;
  pub(crate) struct LazyCompilationDependencyFactory {
    normal_module_factory: Arc<NormalModuleFactory>,
  }
  #[automatically_derived]
  impl ::core::fmt::Debug for LazyCompilationDependencyFactory {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      ::core::fmt::Formatter::debug_struct_field1_finish(
        f,
        "LazyCompilationDependencyFactory",
        "normal_module_factory",
        &&self.normal_module_factory,
      )
    }
  }
  impl LazyCompilationDependencyFactory {
    pub fn new(normal_module_factory: Arc<NormalModuleFactory>) -> Self {
      Self {
        normal_module_factory,
      }
    }
  }
  impl ModuleFactory for LazyCompilationDependencyFactory {
    #[allow(
      clippy::async_yields_async,
      clippy::diverging_sub_expression,
      clippy::let_unit_value,
      clippy::no_effect_underscore_binding,
      clippy::shadow_same,
      clippy::type_complexity,
      clippy::type_repetition_in_bounds,
      clippy::used_underscore_binding
    )]
    fn create<'life0, 'life1, 'async_trait>(
      &'life0 self,
      data: &'life1 mut ModuleFactoryCreateData,
    ) -> ::core::pin::Pin<
      Box<
        dyn ::core::future::Future<Output = Result<ModuleFactoryResult>>
          + ::core::marker::Send
          + 'async_trait,
      >,
    >
    where
      'life0: 'async_trait,
      'life1: 'async_trait,
      Self: 'async_trait,
    {
      Box::pin(async move {
        if let ::core::option::Option::Some(__ret) =
          ::core::option::Option::None::<Result<ModuleFactoryResult>>
        {
          return __ret;
        }
        let __self = self;
        let __ret: Result<ModuleFactoryResult> = {
          let dep: &LazyCompilationDependency = data
            .dependency
            .as_any()
            .downcast_ref()
            .expect("should be lazy compile dependency");
          let proxy_data = &dep.original_module_create_data;
          let dep = dep.clone();
          let mut create_data = ModuleFactoryCreateData {
            resolve_options: proxy_data.resolve_options.clone(),
            context: proxy_data.context.clone(),
            dependency: Box::new(dep),
            issuer: proxy_data.issuer.clone(),
            issuer_identifier: proxy_data.issuer_identifier,
            file_dependencies: proxy_data.file_dependencies.clone(),
            context_dependencies: proxy_data.context_dependencies.clone(),
            missing_dependencies: proxy_data.missing_dependencies.clone(),
            diagnostics: proxy_data.diagnostics.clone(),
          };
          __self.normal_module_factory.create(&mut create_data).await
        };
        #[allow(unreachable_code)]
        __ret
      })
    }
  }
}
mod module {
  use std::{hash::Hash, path::PathBuf, sync::Arc};

  use rspack_core::{
    impl_build_info_meta, module_namespace_promise,
    rspack_sources::{RawSource, Source},
    AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BuildContext,
    BuildInfo, BuildMeta, BuildResult, CodeGenerationResult, Compilation, ConcatenationScope,
    Context, DependenciesBlock, DependencyId, Module, ModuleIdentifier, ModuleType, RuntimeGlobals,
    RuntimeSpec, SourceType, TemplateContext,
  };
  use rspack_error::{Diagnosable, Diagnostic, Result};
  use rspack_identifier::Identifiable;
  use rspack_plugin_javascript::dependency::CommonJsRequireDependency;
  use rspack_util::source_map::{ModuleSourceMapConfig, SourceMapKind};
  use rustc_hash::FxHashSet;

  use crate::dependency::{LazyCompilationDependency, ProxyCreateData};
  static MODULE_TYPE: ModuleType = ModuleType::Js;
  static SOURCE_TYPE: [SourceType; 1] = [SourceType::JavaScript];
  pub(crate) struct LazyCompilationProxyModule {
    build_info: Option<BuildInfo>,
    build_meta: Option<BuildMeta>,
    original_module: ModuleIdentifier,
    cacheable: bool,
    readable_identifier: String,
    identifier: ModuleIdentifier,
    blocks: Vec<AsyncDependenciesBlockIdentifier>,
    dependencies: Vec<DependencyId>,
    source_map_kind: SourceMapKind,
    create_data: ProxyCreateData,
    pub request: String,
    pub active: bool,
    pub data: String,
    pub client: String,
  }
  #[automatically_derived]
  impl ::core::fmt::Debug for LazyCompilationProxyModule {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      let names: &'static _ = &[
        "build_info",
        "build_meta",
        "original_module",
        "cacheable",
        "readable_identifier",
        "identifier",
        "blocks",
        "dependencies",
        "source_map_kind",
        "create_data",
        "request",
        "active",
        "data",
        "client",
      ];
      let values: &[&dyn ::core::fmt::Debug] = &[
        &self.build_info,
        &self.build_meta,
        &self.original_module,
        &self.cacheable,
        &self.readable_identifier,
        &self.identifier,
        &self.blocks,
        &self.dependencies,
        &self.source_map_kind,
        &self.create_data,
        &self.request,
        &self.active,
        &self.data,
        &&self.client,
      ];
      ::core::fmt::Formatter::debug_struct_fields_finish(
        f,
        "LazyCompilationProxyModule",
        names,
        values,
      )
    }
  }
  impl Hash for LazyCompilationProxyModule {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
      self.build_meta.hash(state);
      self.original_module.hash(state);
      self.readable_identifier.hash(state);
      self.identifier.hash(state);
      self.blocks.hash(state);
      self.dependencies.hash(state);
    }
  }
  impl PartialEq for LazyCompilationProxyModule {
    fn eq(&self, other: &Self) -> bool {
      self.original_module == other.original_module
        && self.readable_identifier == other.readable_identifier
        && self.identifier == other.identifier
    }
  }
  impl Eq for LazyCompilationProxyModule {}
  impl ModuleSourceMapConfig for LazyCompilationProxyModule {
    fn get_source_map_kind(&self) -> &SourceMapKind {
      &self.source_map_kind
    }
    fn set_source_map_kind(&mut self, source_map: SourceMapKind) {
      self.source_map_kind = source_map;
    }
  }
  impl LazyCompilationProxyModule {
    pub(crate) fn new(
      original_module: ModuleIdentifier,
      create_data: ProxyCreateData,
      request: String,
      cacheable: bool,
      active: bool,
      data: String,
      client: String,
    ) -> Self {
      let readable_identifier = {
        let res = ::alloc::fmt::format(format_args!(
          "lazy-compilation-proxy|{0}",
          create_data.context.shorten(&original_module)
        ));
        res
      };
      let identifier = {
        let res = ::alloc::fmt::format(format_args!("lazy-compilation-proxy|{0}", original_module));
        res
      }
      .into();
      Self {
        build_info: None,
        build_meta: None,
        cacheable,
        original_module,
        create_data,
        readable_identifier,
        identifier,
        source_map_kind: SourceMapKind::None,
        blocks: ::alloc::vec::Vec::new(),
        dependencies: ::alloc::vec::Vec::new(),
        active,
        request,
        client,
        data,
      }
    }
  }
  impl Diagnosable for LazyCompilationProxyModule {
    fn add_diagnostic(&self, _diagnostic: Diagnostic) {
      ::core::panicking::panic("not implemented")
    }
    fn add_diagnostics(&self, _diagnostics: Vec<Diagnostic>) {
      ::core::panicking::panic("not implemented")
    }
  }
  impl Module for LazyCompilationProxyModule {
    fn build_info(&self) -> Option<&::rspack_core::BuildInfo> {
      self.build_info.as_ref()
    }
    fn build_meta(&self) -> Option<&::rspack_core::BuildMeta> {
      self.build_meta.as_ref()
    }
    fn set_module_build_info_and_meta(
      &mut self,
      build_info: ::rspack_core::BuildInfo,
      build_meta: ::rspack_core::BuildMeta,
    ) {
      self.build_info = Some(build_info);
      self.build_meta = Some(build_meta);
    }
    fn source_types(&self) -> &[SourceType] {
      &SOURCE_TYPE
    }
    fn module_type(&self) -> &ModuleType {
      &MODULE_TYPE
    }
    fn size(&self, _source_type: &SourceType) -> f64 {
      200f64
    }
    fn original_source(&self) -> Option<&dyn Source> {
      None
    }
    fn readable_identifier(&self, _context: &Context) -> std::borrow::Cow<str> {
      std::borrow::Cow::Borrowed(&self.readable_identifier)
    }
    fn get_diagnostics(&self) -> Vec<Diagnostic> {
      ::alloc::vec::Vec::new()
    }
    #[allow(
      clippy::async_yields_async,
      clippy::diverging_sub_expression,
      clippy::let_unit_value,
      clippy::no_effect_underscore_binding,
      clippy::shadow_same,
      clippy::type_complexity,
      clippy::type_repetition_in_bounds,
      clippy::used_underscore_binding
    )]
    fn build<'life0, 'life1, 'life2, 'async_trait>(
      &'life0 mut self,
      _build_context: BuildContext<'life1>,
      _compilation: Option<&'life2 Compilation>,
    ) -> ::core::pin::Pin<
      Box<
        dyn ::core::future::Future<Output = Result<BuildResult>>
          + ::core::marker::Send
          + 'async_trait,
      >,
    >
    where
      'life0: 'async_trait,
      'life1: 'async_trait,
      'life2: 'async_trait,
      Self: 'async_trait,
    {
      Box::pin(async move {
        if let ::core::option::Option::Some(__ret) =
          ::core::option::Option::None::<Result<BuildResult>>
        {
          return __ret;
        }
        let mut __self = self;
        let _build_context = _build_context;
        let _compilation = _compilation;
        let __ret: Result<BuildResult> = {
          let client_dep = CommonJsRequireDependency::new(__self.client.clone(), None, 0, 0, false);
          let mut dependencies = ::alloc::vec::Vec::new();
          let mut blocks = ::alloc::vec::Vec::new();
          dependencies.push(Box::new(client_dep) as BoxDependency);
          if __self.active {
            let dep = LazyCompilationDependency::new(__self.create_data.clone());
            blocks.push(AsyncDependenciesBlock::new(
              __self.identifier,
              None,
              None,
              <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([Box::new(dep)]),
              ),
            ));
          }
          let mut files = FxHashSet::default();
          files.extend(__self.create_data.file_dependencies.clone());
          files.insert(PathBuf::from(
            &__self.create_data.resource_resolve_data.resource,
          ));
          Ok(BuildResult {
            build_info: BuildInfo {
              cacheable: __self.cacheable,
              file_dependencies: files,
              ..Default::default()
            },
            build_meta: BuildMeta::default(),
            analyze_result: Default::default(),
            dependencies,
            blocks,
            optimization_bailouts: ::alloc::vec::Vec::new(),
          })
        };
        #[allow(unreachable_code)]
        __ret
      })
    }
    fn code_generation(
      &self,
      compilation: &Compilation,
      _runtime: Option<&RuntimeSpec>,
      mut concatenation_scope: Option<ConcatenationScope>,
    ) -> Result<CodeGenerationResult> {
      let mut runtime_requirements = RuntimeGlobals::empty();
      runtime_requirements.insert(RuntimeGlobals::MODULE);
      runtime_requirements.insert(RuntimeGlobals::REQUIRE);
      let client_dep_id = self.dependencies[0];
      let module_graph = &compilation.get_module_graph();
      let chunk_graph = &compilation.chunk_graph;
      let client_module = module_graph
        .module_identifier_by_dependency_id(&client_dep_id)
        .expect("should have module");
      let block = self.blocks.first();
      let client = {
        let res = ::alloc::fmt::format(format_args!(
          "var client = __webpack_require__(\"{0}\");\nvar data = \"{1}\"",
          chunk_graph
            .get_module_id(*client_module)
            .as_ref()
            .expect("should have module id"),
          self.data
        ));
        res
      };
      let keep_active = {
        let res = ::alloc::fmt::format(
                    format_args!(
                        "var dispose = client.keepAlive({{ data: data, active: {0}, module: module, onError: onError }})",
                        block.is_some()
                    ),
                );
        res
      };
      let source = if let Some(block_id) = block {
        let block = module_graph
          .block_by_id(block_id)
          .expect("should have block");
        let dep_id = block.get_dependencies()[0];
        let module = module_graph
          .module_identifier_by_dependency_id(&dep_id)
          .expect("should have module");
        let mut template_ctx = TemplateContext {
          compilation,
          module: module_graph
            .module_by_identifier(module)
            .expect("should have module")
            .as_ref(),
          runtime_requirements: &mut runtime_requirements,
          init_fragments: &mut ::alloc::vec::Vec::new(),
          runtime: None,
          concatenation_scope: concatenation_scope.as_mut(),
        };
        RawSource::from({
          let res = ::alloc::fmt::format(
                        format_args!(
                            "{3}\n        module.exports = {0};\n        if (module.hot) {{\n          module.hot.accept();\n          module.hot.accept(\"{1}\", function() {{ module.hot.invalidate(); }});\n          module.hot.dispose(function(data) {{ delete data.resolveSelf; dispose(data); }});\n          if (module.hot.data && module.hot.data.resolveSelf)\n            module.hot.data.resolveSelf(module.exports);\n        }}\n        function onError() {{ /* ignore */ }}\n        {2}\n        ",
                            module_namespace_promise(& mut template_ctx, & dep_id,
                            Some(block_id), & self.request, "import()", false),
                            chunk_graph.get_module_id(* module).as_ref()
                            .expect("should have module id"), keep_active, client
                        ),
                    );
          res
        })
      } else {
        RawSource::from({
          let res = ::alloc::fmt::format(
                        format_args!(
                            "{0}\n        var resolveSelf, onError;\n        module.exports = new Promise(function(resolve, reject) {{ resolveSelf = resolve; onError = reject; }});\n        if (module.hot) {{\n          module.hot.accept();\n          if (module.hot.data && module.hot.data.resolveSelf) module.hot.data.resolveSelf(module.exports);\n          module.hot.dispose(function(data) {{ data.resolveSelf = resolveSelf; dispose(data); }});\n        }}\n        {1}\n      ",
                            client, keep_active
                        ),
                    );
          res
        })
      };
      let mut codegen_result = CodeGenerationResult::default().with_javascript(Arc::new(source));
      codegen_result.runtime_requirements = runtime_requirements;
      codegen_result.set_hash(
        &compilation.options.output.hash_function,
        &compilation.options.output.hash_digest,
        &compilation.options.output.hash_salt,
      );
      Ok(codegen_result)
    }
  }
  impl Identifiable for LazyCompilationProxyModule {
    fn identifier(&self) -> rspack_identifier::Identifier {
      self.identifier
    }
  }
  impl DependenciesBlock for LazyCompilationProxyModule {
    fn add_block_id(&mut self, block: rspack_core::AsyncDependenciesBlockIdentifier) {
      self.blocks.push(block);
    }
    fn get_blocks(&self) -> &[rspack_core::AsyncDependenciesBlockIdentifier] {
      &self.blocks
    }
    fn add_dependency_id(&mut self, dependency: rspack_core::DependencyId) {
      self.dependencies.push(dependency);
    }
    fn get_dependencies(&self) -> &[rspack_core::DependencyId] {
      &self.dependencies
    }
  }
}
pub mod plugin {
  use std::sync::Arc;

  use once_cell::sync::Lazy;
  use rspack_core::{
    BoxModule, Compilation, CompilationParams, DependencyType, ModuleFactory,
    NormalModuleCreateData, Plugin, PluginContext, PluginNormalModuleFactoryModuleHookOutput,
  };
  use rspack_hook::{plugin, plugin_hook, AsyncSeries2};
  use rspack_regex::RspackRegex;
  use tokio::sync::Mutex;

  use crate::{
    backend::Backend, dependency::ProxyCreateData, factory::LazyCompilationDependencyFactory,
    module::LazyCompilationProxyModule,
  };
  static WEBPACK_DEV_SERVER_CLIENT_RE: Lazy<RspackRegex> = Lazy::new(|| {
    RspackRegex::new(
                r#"(webpack|rspack)[/\\]hot[/\\]|(webpack|rspack)-dev-server[/\\]client|(webpack|rspack)-hot-middleware[/\\]client"#,
            )
            .expect("should compile regex")
  });
  pub struct LazyCompilationPlugin<T: Backend> {
    inner: ::std::sync::Arc<LazyCompilationPluginInner<T>>,
  }
  #[automatically_derived]
  impl<T: ::core::fmt::Debug + Backend> ::core::fmt::Debug for LazyCompilationPlugin<T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      ::core::fmt::Formatter::debug_struct_field1_finish(
        f,
        "LazyCompilationPlugin",
        "inner",
        &&self.inner,
      )
    }
  }
  impl<T: Backend> LazyCompilationPlugin<T> {
    #[allow(clippy::too_many_arguments)]
    fn new_inner(
      backend: Mutex<T>,
      entries: bool,
      imports: bool,
      test: Option<RspackRegex>,
      cacheable: bool,
    ) -> Self {
      Self {
        inner: ::std::sync::Arc::new(LazyCompilationPluginInner {
          backend,
          entries,
          imports,
          test,
          cacheable,
        }),
      }
    }
    fn from_inner(inner: &::std::sync::Arc<LazyCompilationPluginInner<T>>) -> Self {
      Self {
        inner: ::std::sync::Arc::clone(inner),
      }
    }
    fn inner(&self) -> &::std::sync::Arc<LazyCompilationPluginInner<T>> {
      &self.inner
    }
  }
  impl<T: Backend> ::std::ops::Deref for LazyCompilationPlugin<T> {
    type Target = LazyCompilationPluginInner<T>;
    fn deref(&self) -> &Self::Target {
      &self.inner
    }
  }
  #[doc(hidden)]
  pub struct LazyCompilationPluginInner<T: Backend> {
    backend: Mutex<T>,
    entries: bool,
    imports: bool,
    test: Option<RspackRegex>,
    cacheable: bool,
  }
  #[automatically_derived]
  impl<T: ::core::fmt::Debug + Backend> ::core::fmt::Debug for LazyCompilationPluginInner<T> {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
      ::core::fmt::Formatter::debug_struct_field5_finish(
        f,
        "LazyCompilationPluginInner",
        "backend",
        &self.backend,
        "entries",
        &self.entries,
        "imports",
        &self.imports,
        "test",
        &self.test,
        "cacheable",
        &&self.cacheable,
      )
    }
  }
  impl<T: Backend> LazyCompilationPlugin<T> {
    pub fn new(
      cacheable: bool,
      backend: T,
      test: Option<RspackRegex>,
      entries: bool,
      imports: bool,
    ) -> Self {
      Self::new_inner(Mutex::new(backend), entries, imports, test, cacheable)
    }
    fn check_test(&self, module: &BoxModule) -> bool {
      if let Some(test) = &self.test {
        test.test(&module.name_for_condition().unwrap_or("".into()))
      } else {
        true
      }
    }
  }
  #[allow(non_camel_case_types)]
  struct compilation<T: Backend> {
    inner: ::std::sync::Arc<LazyCompilationPluginInner<T>>,
  }
  impl<T: Backend> compilation<T> {
    pub(crate) fn new(plugin: &LazyCompilationPlugin<T>) -> Box<Self> {
      Box::new(compilation {
        inner: ::std::sync::Arc::clone(plugin.inner()),
      })
    }
  }
  impl<T: Backend> LazyCompilationPlugin<T> {
    #[allow(clippy::ptr_arg)]
    async fn compilation(
      &self,
      compilation: &mut Compilation,
      params: &mut CompilationParams,
    ) -> Result<()> {
      Ok(())
    }
  }
  impl<T: Backend> ::std::ops::Deref for compilation<T> {
    type Target = LazyCompilationPluginInner<T>;
    fn deref(&self) -> &Self::Target {
      &self.inner
    }
  }
  impl<T: Backend> AsyncSeries2<Compilation, CompilationParams> for compilation<T> {
    #[allow(
      clippy::async_yields_async,
      clippy::diverging_sub_expression,
      clippy::let_unit_value,
      clippy::no_effect_underscore_binding,
      clippy::shadow_same,
      clippy::type_complexity,
      clippy::type_repetition_in_bounds,
      clippy::used_underscore_binding
    )]
    fn run<'life0, 'life1, 'life2, 'async_trait>(
      &'life0 self,
      compilation: &'life1 mut Compilation,
      params: &'life2 mut CompilationParams,
    ) -> ::core::pin::Pin<
      Box<dyn ::core::future::Future<Output = Result<()>> + ::core::marker::Send + 'async_trait>,
    >
    where
      'life0: 'async_trait,
      'life1: 'async_trait,
      'life2: 'async_trait,
      Self: 'async_trait,
    {
      Box::pin(async move {
        if let ::core::option::Option::Some(__ret) = ::core::option::Option::None::<Result<()>> {
          return __ret;
        }
        let __self = self;
        let __ret: Result<()> = {
          LazyCompilationPlugin::compilation(
            &LazyCompilationPlugin::from_inner(&__self.inner),
            compilation,
            params,
          )
          .await
        };
        #[allow(unreachable_code)]
        __ret
      })
    }
  }
  impl<T: Backend> Plugin for LazyCompilationPlugin<T> {
    #[allow(
      clippy::async_yields_async,
      clippy::diverging_sub_expression,
      clippy::let_unit_value,
      clippy::no_effect_underscore_binding,
      clippy::shadow_same,
      clippy::type_complexity,
      clippy::type_repetition_in_bounds,
      clippy::used_underscore_binding
    )]
    fn normal_module_factory_module<'life0, 'life1, 'life2, 'async_trait>(
      &'life0 self,
      _ctx: PluginContext,
      module: BoxModule,
      args: &'life1 mut NormalModuleCreateData<'life2>,
    ) -> ::core::pin::Pin<
      Box<
        dyn ::core::future::Future<Output = PluginNormalModuleFactoryModuleHookOutput>
          + ::core::marker::Send
          + 'async_trait,
      >,
    >
    where
      'life0: 'async_trait,
      'life1: 'async_trait,
      'life2: 'async_trait,
      Self: 'async_trait,
    {
      Box::pin(async move {
        if let ::core::option::Option::Some(__ret) =
          ::core::option::Option::None::<PluginNormalModuleFactoryModuleHookOutput>
        {
          return __ret;
        }
        let __self = self;
        let _ctx = _ctx;
        let module = module;
        let __ret: PluginNormalModuleFactoryModuleHookOutput = {
          if let Some(query) = &args.resource_resolve_data.resource_query
            && query.contains("lazy-compilation-proxy-dep")
          {
            let remaining_query = query.clone().replace("lazy-compilation-proxy-dep", "");
            args.resource_resolve_data.resource_query =
              if remaining_query.is_empty() || remaining_query == "?" {
                None
              } else {
                Some(remaining_query)
              };
            return Ok(module);
          }
          let create_data = args.create_data;
          let dep_type = create_data.dependency.dependency_type();
          let is_imports = match dep_type {
            DependencyType::DynamicImport
            | DependencyType::DynamicImportEager
            | DependencyType::ContextElement => true,
            _ => false,
          };
          let is_entries = match dep_type {
            DependencyType::Entry => true,
            _ => false,
          };
          #[allow(clippy::if_same_then_else)]
          if match dep_type {
            DependencyType::ModuleHotAccept
            | DependencyType::ModuleHotDecline
            | DependencyType::ImportMetaHotAccept
            | DependencyType::ImportMetaHotDecline => true,
            _ => false,
          } {
            return Ok(module);
          } else if !is_entries && !is_imports {
            return Ok(module);
          }
          if !__self.entries && is_entries {
            return Ok(module);
          }
          if !__self.imports && is_imports {
            return Ok(module);
          }
          if WEBPACK_DEV_SERVER_CLIENT_RE.test(args.resolve_data_request)
            || !__self.check_test(&module)
          {
            return Ok(module);
          }
          let mut backend = __self.backend.lock().await;
          let module_identifier = module.identifier();
          let info = backend
            .module(
              module_identifier,
              args.resource_resolve_data.resource.clone(),
            )
            .await?;
          match module_identifier {
            tmp => {
              {
                ::std::io::_eprint(format_args!(
                  "[{0}:{1}:{2}] {3} = {4:#?}\n",
                  "crates/rspack_plugin_lazy_compilation/src/plugin.rs",
                  151u32,
                  5u32,
                  "module_identifier",
                  &tmp
                ));
              };
              tmp
            }
          };
          Ok(Box::new(LazyCompilationProxyModule::new(
            module_identifier,
            ProxyCreateData::new(args),
            args.resolve_data_request.to_string(),
            __self.cacheable,
            info.active,
            info.data,
            info.client,
          )) as BoxModule)
        };
        #[allow(unreachable_code)]
        __ret
      })
    }
  }
}
