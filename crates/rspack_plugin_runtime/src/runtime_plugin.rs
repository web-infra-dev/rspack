use std::{
  hash::Hash,
  sync::{Arc, LazyLock},
};

use atomic_refcell::AtomicRefCell;
use rspack_collections::DatabaseItem;
use rspack_core::{
  ChunkLoading, ChunkUkey, Compilation, CompilationId, CompilationParams,
  CompilationRuntimeRequirementInModule, CompilationRuntimeRequirementInTree, CompilerCompilation,
  MODULE_GLOBALS, ModuleIdentifier, Plugin, PublicPath, REQUIRE_SCOPE_GLOBALS, RuntimeGlobals,
  RuntimeModule, RuntimeModuleExt, SourceType, get_css_chunk_filename_template,
  get_js_chunk_filename_template,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JsPlugin, impl_plugin_for_js_plugin::chunk_has_js,
};
#[cfg(allocative)]
use rspack_util::allocative;
use rspack_util::fx_hash::FxDashMap;

use crate::{
  RuntimePluginHooks,
  runtime_module::{
    AmdDefineRuntimeModule, AmdOptionsRuntimeModule, AsyncRuntimeModule,
    AutoPublicPathRuntimeModule, BaseUriRuntimeModule, ChunkNameRuntimeModule,
    ChunkPrefetchPreloadFunctionRuntimeModule, CompatGetDefaultExportRuntimeModule,
    CreateFakeNamespaceObjectRuntimeModule, CreateScriptRuntimeModule,
    CreateScriptUrlRuntimeModule, DefinePropertyGettersRuntimeModule,
    ESMModuleDecoratorRuntimeModule, EnsureChunkRuntimeModule, GetChunkFilenameRuntimeModule,
    GetChunkUpdateFilenameRuntimeModule, GetFullHashRuntimeModule, GetMainFilenameRuntimeModule,
    GetTrustedTypesPolicyRuntimeModule, GlobalRuntimeModule, HasOwnPropertyRuntimeModule,
    LoadScriptRuntimeModule, MakeDeferredNamespaceObjectRuntimeModule,
    MakeNamespaceObjectRuntimeModule, MakeOptimizedDeferredNamespaceObjectRuntimeModule,
    NodeModuleDecoratorRuntimeModule, NonceRuntimeModule, OnChunkLoadedRuntimeModule,
    PublicPathRuntimeModule, RelativeUrlRuntimeModule, RuntimeIdRuntimeModule,
    SystemContextRuntimeModule, ToBinaryRuntimeModule, chunk_has_css, is_enabled_for_chunk,
  },
};

/// Safety with [atomic_refcell::AtomicRefCell]:
///
/// We should make sure that there's no read-write and write-write conflicts for each hook instance by looking up [RuntimePlugin::get_compilation_hooks_mut]
type ArcRuntimePluginHooks = Arc<AtomicRefCell<RuntimePluginHooks>>;

#[cfg_attr(allocative, allocative::root)]
static COMPILATION_HOOKS_MAP: LazyLock<FxDashMap<CompilationId, ArcRuntimePluginHooks>> =
  LazyLock::new(Default::default);

const MODULE_DEPENDENCIES: &[(RuntimeGlobals, RuntimeGlobals)] = &[
  (
    RuntimeGlobals::ESM_MODULE_DECORATOR,
    RuntimeGlobals::MODULE.union(RuntimeGlobals::MODULE_ID),
  ),
  (
    RuntimeGlobals::NODE_MODULE_DECORATOR,
    RuntimeGlobals::MODULE,
  ),
  (RuntimeGlobals::AMD_DEFINE, RuntimeGlobals::REQUIRE),
];

fn handle_scope_globals(
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) {
  if runtime_requirements
    .iter()
    .any(|requirement| REQUIRE_SCOPE_GLOBALS.contains(requirement))
  {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE_SCOPE);
  }
  if runtime_requirements
    .iter()
    .any(|requirement| MODULE_GLOBALS.contains(requirement))
  {
    runtime_requirements_mut.insert(RuntimeGlobals::MODULE);
  }
}

fn handle_dependency_globals(
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  dependencies: &[(RuntimeGlobals, RuntimeGlobals)],
) {
  for &(requirement, dependencies) in dependencies.iter() {
    if runtime_requirements.contains(requirement) {
      *runtime_requirements_mut |= dependencies;
    }
  }
}

#[plugin]
#[derive(Debug, Default)]
pub struct RuntimePlugin;

impl RuntimePlugin {
  pub fn get_compilation_hooks(id: CompilationId) -> ArcRuntimePluginHooks {
    if !COMPILATION_HOOKS_MAP.contains_key(&id) {
      COMPILATION_HOOKS_MAP.insert(id, Default::default());
    }
    COMPILATION_HOOKS_MAP
      .get(&id)
      .expect("should have js plugin drive")
      .clone()
  }

  pub fn get_compilation_hooks_mut(id: CompilationId) -> ArcRuntimePluginHooks {
    COMPILATION_HOOKS_MAP.entry(id).or_default().clone()
  }
}

#[plugin_hook(CompilerCompilation for RuntimePlugin)]
async fn compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let hooks = JsPlugin::get_compilation_hooks_mut(compilation.id());
  let mut hooks = hooks.write().await;
  hooks.chunk_hash.tap(js_chunk_hash::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for RuntimePlugin)]
async fn js_chunk_hash(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  for identifier in compilation
    .build_chunk_graph_artifact
    .chunk_graph
    .get_chunk_runtime_modules_iterable(chunk_ukey)
  {
    if let Some(hash) = compilation.runtime_modules_hash.get(identifier) {
      hash.hash(hasher);
    }
  }
  Ok(())
}

#[plugin_hook(CompilationRuntimeRequirementInModule for RuntimePlugin,tracing=false)]
async fn runtime_requirements_in_module(
  &self,
  _compilation: &Compilation,
  _module: &ModuleIdentifier,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) -> Result<Option<()>> {
  handle_scope_globals(runtime_requirements, runtime_requirements_mut);
  handle_dependency_globals(
    runtime_requirements,
    runtime_requirements_mut,
    MODULE_DEPENDENCIES,
  );

  Ok(None)
}

#[plugin_hook(CompilationRuntimeRequirementInTree for RuntimePlugin)]
async fn runtime_requirements_in_tree(
  &self,
  compilation: &Compilation,
  chunk_ukey: &ChunkUkey,
  _all_runtime_requirements: &RuntimeGlobals,
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  runtime_modules_to_add: &mut Vec<(ChunkUkey, Box<dyn RuntimeModule>)>,
) -> Result<Option<()>> {
  handle_scope_globals(runtime_requirements, runtime_requirements_mut);

  if runtime_requirements.contains(RuntimeGlobals::INITIALIZE_SHARING) {
    runtime_requirements_mut.insert(RuntimeGlobals::SHARE_SCOPE_MAP);
  }

  if runtime_requirements.contains(RuntimeGlobals::SHARE_SCOPE_MAP) {
    runtime_requirements_mut.insert(RuntimeGlobals::HAS_OWN_PROPERTY);
  }

  if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES) {
    runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
  }

  if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK) {
    let c = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    let has_async_chunks =
      c.has_async_chunks(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey);
    runtime_modules_to_add.push((
      *chunk_ukey,
      EnsureChunkRuntimeModule::new(&compilation.runtime_template, has_async_chunks).boxed(),
    ));
  }

  let library_type = {
    let chunk = compilation
      .build_chunk_graph_artifact
      .chunk_by_ukey
      .expect_get(chunk_ukey);
    chunk
      .get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
      .and_then(|options| options.library.as_ref())
      .or(compilation.options.output.library.as_ref())
      .map(|library| library.library_type.clone())
  };

  for runtime_requirement in runtime_requirements.iter() {
    match runtime_requirement {
      RuntimeGlobals::ASYNC_MODULE => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          AsyncRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::BASE_URI
        if is_enabled_for_chunk(chunk_ukey, &ChunkLoading::Disable, compilation) =>
      {
        runtime_modules_to_add.push((
          *chunk_ukey,
          BaseUriRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::PUBLIC_PATH => {
        let public_path = compilation
          .build_chunk_graph_artifact
          .chunk_by_ukey
          .expect_get(chunk_ukey)
          .get_entry_options(&compilation.build_chunk_graph_artifact.chunk_group_by_ukey)
          .and_then(|options| options.public_path.clone())
          .unwrap_or_else(|| compilation.options.output.public_path.clone());
        match &public_path {
          PublicPath::Filename(filename) => {
            runtime_modules_to_add.push((
              *chunk_ukey,
              PublicPathRuntimeModule::new(
                &compilation.runtime_template,
                Box::new(filename.clone()),
              )
              .boxed(),
            ));
          }
          PublicPath::Auto => {
            runtime_modules_to_add.push((
              *chunk_ukey,
              AutoPublicPathRuntimeModule::new(&compilation.runtime_template).boxed(),
            ));
          }
        }
      }
      RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetChunkFilenameRuntimeModule::new(
            &compilation.runtime_template,
            "javascript",
            "javascript",
            SourceType::JavaScript,
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME),
            |_| false,
            |chunk, compilation| {
              chunk_has_js(&chunk.ukey(), compilation).then(|| {
                get_js_chunk_filename_template(
                  chunk,
                  &compilation.options.output,
                  &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
                )
              })
            },
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::GET_CHUNK_CSS_FILENAME => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetChunkFilenameRuntimeModule::new(
            &compilation.runtime_template,
            "css",
            "css",
            SourceType::Css,
            compilation
              .runtime_template
              .render_runtime_globals(&RuntimeGlobals::GET_CHUNK_CSS_FILENAME),
            |runtime_requirements| {
              runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
            },
            |chunk, compilation| {
              chunk_has_css(&chunk.ukey(), compilation).then(|| {
                get_css_chunk_filename_template(
                  chunk,
                  &compilation.options.output,
                  &compilation.build_chunk_graph_artifact.chunk_group_by_ukey,
                )
                .clone()
              })
            },
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetChunkUpdateFilenameRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetMainFilenameRuntimeModule::new(
            &compilation.runtime_template,
            "update manifest",
            RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME,
            compilation.options.output.hot_update_main_filename.clone(),
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::LOAD_SCRIPT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          LoadScriptRuntimeModule::new(
            &compilation.runtime_template,
            compilation.options.output.unique_name.clone(),
            compilation.options.output.trusted_types.is_some(),
            *chunk_ukey,
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::HAS_OWN_PROPERTY => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          HasOwnPropertyRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::GET_FULL_HASH => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetFullHashRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::GLOBAL => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GlobalRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::CREATE_SCRIPT_URL => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          CreateScriptUrlRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::CREATE_SCRIPT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          CreateScriptRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::ON_CHUNKS_LOADED => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          OnChunkLoadedRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::DEFINE_PROPERTY_GETTERS => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          DefinePropertyGettersRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          GetTrustedTypesPolicyRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          CreateFakeNamespaceObjectRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::MAKE_NAMESPACE_OBJECT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          MakeNamespaceObjectRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          CompatGetDefaultExportRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::ESM_MODULE_DECORATOR => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          ESMModuleDecoratorRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::NODE_MODULE_DECORATOR => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          NodeModuleDecoratorRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::SYSTEM_CONTEXT if matches!(&library_type, Some(t) if t == "system") => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          SystemContextRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::SCRIPT_NONCE => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          NonceRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::RELATIVE_URL => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          RelativeUrlRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::CHUNK_NAME => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          ChunkNameRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::RUNTIME_ID => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          RuntimeIdRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      RuntimeGlobals::PREFETCH_CHUNK => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          ChunkPrefetchPreloadFunctionRuntimeModule::new(
            &compilation.runtime_template,
            "prefetch",
            RuntimeGlobals::PREFETCH_CHUNK,
            RuntimeGlobals::PREFETCH_CHUNK_HANDLERS,
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::PRELOAD_CHUNK => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          ChunkPrefetchPreloadFunctionRuntimeModule::new(
            &compilation.runtime_template,
            "preload",
            RuntimeGlobals::PRELOAD_CHUNK,
            RuntimeGlobals::PRELOAD_CHUNK_HANDLERS,
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::AMD_DEFINE => {
        if compilation.options.amd.is_some() {
          runtime_modules_to_add.push((
            *chunk_ukey,
            AmdDefineRuntimeModule::new(&compilation.runtime_template).boxed(),
          ));
        }
      }
      RuntimeGlobals::AMD_OPTIONS => {
        if let Some(options) = &compilation.options.amd {
          runtime_modules_to_add.push((
            *chunk_ukey,
            AmdOptionsRuntimeModule::new(&compilation.runtime_template, options.clone()).boxed(),
          ));
        }
      }
      RuntimeGlobals::MAKE_DEFERRED_NAMESPACE_OBJECT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          MakeDeferredNamespaceObjectRuntimeModule::new(&compilation.runtime_template, *chunk_ukey)
            .boxed(),
        ));
      }
      RuntimeGlobals::MAKE_OPTIMIZED_DEFERRED_NAMESPACE_OBJECT => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          MakeOptimizedDeferredNamespaceObjectRuntimeModule::new(
            &compilation.runtime_template,
            *chunk_ukey,
          )
          .boxed(),
        ));
      }
      RuntimeGlobals::TO_BINARY => {
        runtime_modules_to_add.push((
          *chunk_ukey,
          ToBinaryRuntimeModule::new(&compilation.runtime_template).boxed(),
        ));
      }
      _ => {}
    }
  }

  Ok(None)
}

impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.RuntimePlugin"
  }

  fn apply(&self, ctx: &mut rspack_core::ApplyContext<'_>) -> Result<()> {
    ctx.compiler_hooks.compilation.tap(compilation::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_module
      .tap(runtime_requirements_in_module::new(self));
    ctx
      .compilation_hooks
      .runtime_requirement_in_tree
      .tap(runtime_requirements_in_tree::new(self));
    Ok(())
  }

  fn clear_cache(&self, id: CompilationId) {
    COMPILATION_HOOKS_MAP.remove(&id);
  }
}
