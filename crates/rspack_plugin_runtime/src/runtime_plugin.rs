use std::hash::Hash;

use async_trait::async_trait;
use once_cell::sync::Lazy;
use rspack_core::{
  get_css_chunk_filename_template, get_js_chunk_filename_template,
  AdditionalChunkRuntimeRequirementsArgs, AdditionalModuleRequirementsArgs, ChunkLoading,
  JsChunkHashArgs, Plugin, PluginAdditionalChunkRuntimeRequirementsOutput,
  PluginAdditionalModuleRequirementsOutput, PluginContext, PluginJsChunkHashHookOutput,
  PluginRuntimeRequirementsInTreeOutput, PublicPath, RuntimeGlobals, RuntimeModuleExt,
  RuntimeRequirementsInTreeArgs, SourceType, FULL_HASH_PLACEHOLDER, HASH_PLACEHOLDER,
};

use crate::runtime_module::{
  chunk_has_css, is_enabled_for_chunk, AsyncRuntimeModule, AutoPublicPathRuntimeModule,
  BaseUriRuntimeModule, ChunkNameRuntimeModule, ChunkPrefetchPreloadFunctionRuntimeModule,
  CompatGetDefaultExportRuntimeModule, CreateFakeNamespaceObjectRuntimeModule,
  CreateScriptUrlRuntimeModule, DefinePropertyGettersRuntimeModule, EnsureChunkRuntimeModule,
  GetChunkFilenameRuntimeModule, GetChunkUpdateFilenameRuntimeModule, GetFullHashRuntimeModule,
  GetMainFilenameRuntimeModule, GetTrustedTypesPolicyRuntimeModule, GlobalRuntimeModule,
  HarmonyModuleDecoratorRuntimeModule, HasOwnPropertyRuntimeModule, LoadScriptRuntimeModule,
  MakeNamespaceObjectRuntimeModule, NodeModuleDecoratorRuntimeModule, NonceRuntimeModule,
  NormalRuntimeModule, OnChunkLoadedRuntimeModule, PublicPathRuntimeModule,
  RelativeUrlRuntimeModule, RuntimeIdRuntimeModule, SystemContextRuntimeModule,
};

static GLOBALS_ON_REQUIRE: Lazy<Vec<RuntimeGlobals>> = Lazy::new(|| {
  vec![
    RuntimeGlobals::CHUNK_NAME,
    RuntimeGlobals::RUNTIME_ID,
    RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
    RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
    RuntimeGlobals::CREATE_SCRIPT,
    RuntimeGlobals::CREATE_SCRIPT_URL,
    RuntimeGlobals::GET_TRUSTED_TYPES_POLICY,
    RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
    RuntimeGlobals::ENSURE_CHUNK,
    RuntimeGlobals::ENTRY_MODULE_ID,
    RuntimeGlobals::GET_FULL_HASH,
    RuntimeGlobals::GLOBAL,
    RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
    RuntimeGlobals::MODULE_CACHE,
    RuntimeGlobals::MODULE_FACTORIES,
    RuntimeGlobals::MODULE_FACTORIES_ADD_ONLY,
    RuntimeGlobals::INTERCEPT_MODULE_EXECUTION,
    RuntimeGlobals::PUBLIC_PATH,
    RuntimeGlobals::BASE_URI,
    RuntimeGlobals::RELATIVE_URL,
    RuntimeGlobals::SCRIPT_NONCE,
    // RuntimeGlobals::UNCAUGHT_ERROR_HANDLER,
    RuntimeGlobals::ASYNC_MODULE,
    // RuntimeGlobals::WASM_INSTANCES,
    RuntimeGlobals::INSTANTIATE_WASM,
    RuntimeGlobals::SHARE_SCOPE_MAP,
    RuntimeGlobals::INITIALIZE_SHARING,
    RuntimeGlobals::LOAD_SCRIPT,
    RuntimeGlobals::SYSTEM_CONTEXT,
    RuntimeGlobals::ON_CHUNKS_LOADED,
  ]
});

static MODULE_DEPENDENCIES: Lazy<Vec<(RuntimeGlobals, Vec<RuntimeGlobals>)>> = Lazy::new(|| {
  vec![
    (RuntimeGlobals::MODULE_LOADED, vec![RuntimeGlobals::MODULE]),
    (RuntimeGlobals::MODULE_ID, vec![RuntimeGlobals::MODULE]),
    (
      RuntimeGlobals::HARMONY_MODULE_DECORATOR,
      vec![RuntimeGlobals::MODULE, RuntimeGlobals::REQUIRE_SCOPE],
    ),
    (
      RuntimeGlobals::NODE_MODULE_DECORATOR,
      vec![RuntimeGlobals::MODULE, RuntimeGlobals::REQUIRE_SCOPE],
    ),
  ]
});

static TREE_DEPENDENCIES: Lazy<Vec<(RuntimeGlobals, Vec<RuntimeGlobals>)>> = Lazy::new(|| {
  vec![
    (
      RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
      vec![RuntimeGlobals::DEFINE_PROPERTY_GETTERS],
    ),
    (
      RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
      vec![
        RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
        RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
        RuntimeGlobals::REQUIRE,
      ],
    ),
    (
      RuntimeGlobals::DEFINE_PROPERTY_GETTERS,
      vec![RuntimeGlobals::HAS_OWN_PROPERTY],
    ),
    (
      RuntimeGlobals::INITIALIZE_SHARING,
      vec![RuntimeGlobals::SHARE_SCOPE_MAP],
    ),
    (
      RuntimeGlobals::SHARE_SCOPE_MAP,
      vec![RuntimeGlobals::HAS_OWN_PROPERTY],
    ),
    (
      RuntimeGlobals::HARMONY_MODULE_DECORATOR,
      vec![RuntimeGlobals::MODULE, RuntimeGlobals::REQUIRE_SCOPE],
    ),
    (
      RuntimeGlobals::NODE_MODULE_DECORATOR,
      vec![RuntimeGlobals::MODULE, RuntimeGlobals::REQUIRE_SCOPE],
    ),
  ]
});

fn handle_require_scope_globals(
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
) {
  if GLOBALS_ON_REQUIRE
    .iter()
    .any(|requirement| runtime_requirements.contains(*requirement))
  {
    runtime_requirements_mut.insert(RuntimeGlobals::REQUIRE_SCOPE);
  }
}

fn handle_dependency_globals(
  runtime_requirements: &RuntimeGlobals,
  runtime_requirements_mut: &mut RuntimeGlobals,
  dependencies: &[(RuntimeGlobals, Vec<RuntimeGlobals>)],
) {
  for (requirement, dependencies) in dependencies.iter() {
    if runtime_requirements.contains(*requirement) {
      runtime_requirements_mut.extend(dependencies.clone());
    }
  }
}

#[derive(Debug)]
pub struct RuntimePlugin;

#[async_trait]
impl Plugin for RuntimePlugin {
  fn name(&self) -> &'static str {
    "rspack.RuntimePlugin"
  }

  async fn additional_tree_runtime_requirements(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalChunkRuntimeRequirementsArgs,
  ) -> PluginAdditionalChunkRuntimeRequirementsOutput {
    let compilation = &args.compilation;
    let chunk = args.chunk();
    if args
      .runtime_requirements
      .contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES)
      || (args
        .runtime_requirements
        .contains(RuntimeGlobals::ENSURE_CHUNK)
        && !chunk
          .get_all_async_chunks(&compilation.chunk_group_by_ukey)
          .is_empty())
    {
      args
        .runtime_requirements
        .insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    }

    Ok(())
  }

  #[allow(clippy::unwrap_in_result)]
  fn runtime_requirements_in_module(
    &self,
    _ctx: PluginContext,
    args: &mut AdditionalModuleRequirementsArgs,
  ) -> PluginAdditionalModuleRequirementsOutput {
    let runtime_requirements = args.runtime_requirements;
    let runtime_requirements_mut = &mut args.runtime_requirements_mut;

    handle_require_scope_globals(runtime_requirements, runtime_requirements_mut);
    handle_dependency_globals(
      runtime_requirements,
      runtime_requirements_mut,
      &MODULE_DEPENDENCIES,
    );

    Ok(())
  }

  #[allow(clippy::unwrap_in_result)]
  async fn runtime_requirements_in_tree(
    &self,
    _ctx: PluginContext,
    args: &mut RuntimeRequirementsInTreeArgs,
  ) -> PluginRuntimeRequirementsInTreeOutput {
    let compilation = &mut args.compilation;
    let chunk = args.chunk;
    let runtime_requirements = args.runtime_requirements;
    let runtime_requirements_mut = &mut args.runtime_requirements_mut;

    if runtime_requirements.contains(RuntimeGlobals::EXPORT_STAR) {
      compilation
        .add_runtime_module(
          chunk,
          NormalRuntimeModule::new(
            RuntimeGlobals::EXPORT_STAR,
            include_str!("runtime/common/_export_star.js"),
          )
          .boxed(),
        )
        .await?
    }

    if compilation.options.output.trusted_types.is_some() {
      if runtime_requirements.contains(RuntimeGlobals::LOAD_SCRIPT) {
        runtime_requirements_mut.insert(RuntimeGlobals::CREATE_SCRIPT_URL);
      }
      if runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT)
        || runtime_requirements.contains(RuntimeGlobals::CREATE_SCRIPT_URL)
      {
        runtime_requirements_mut.insert(RuntimeGlobals::GET_TRUSTED_TYPES_POLICY);
      }
    }

    if (runtime_requirements.contains(RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME)
      && compilation
        .options
        .output
        .hot_update_chunk_filename
        .has_hash_placeholder())
      || (runtime_requirements.contains(RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME)
        && compilation
          .options
          .output
          .hot_update_main_filename
          .has_hash_placeholder())
    {
      runtime_requirements_mut.insert(RuntimeGlobals::GET_FULL_HASH);
    }

    handle_require_scope_globals(runtime_requirements, runtime_requirements_mut);
    handle_dependency_globals(
      runtime_requirements,
      runtime_requirements_mut,
      &TREE_DEPENDENCIES,
    );

    let public_path = compilation
      .chunk_by_ukey
      .expect_get(chunk)
      .get_entry_options(&compilation.chunk_group_by_ukey)
      .and_then(|options| options.public_path.clone())
      .unwrap_or_else(|| compilation.options.output.public_path.clone());

    // TODO check output.scriptType
    if matches!(public_path, PublicPath::Auto)
      && runtime_requirements.contains(RuntimeGlobals::PUBLIC_PATH)
    {
      runtime_requirements_mut.insert(RuntimeGlobals::GLOBAL);
    }

    if runtime_requirements.contains(RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME) {
      let chunk_filename = compilation.options.output.chunk_filename.template();
      if FULL_HASH_PLACEHOLDER.is_match(chunk_filename) || HASH_PLACEHOLDER.is_match(chunk_filename)
      {
        runtime_requirements_mut.insert(RuntimeGlobals::GET_FULL_HASH);
      }
    }

    if runtime_requirements.contains(RuntimeGlobals::GET_CHUNK_CSS_FILENAME) {
      let chunk_filename = compilation.options.output.css_chunk_filename.template();
      if FULL_HASH_PLACEHOLDER.is_match(chunk_filename) || HASH_PLACEHOLDER.is_match(chunk_filename)
      {
        runtime_requirements_mut.insert(RuntimeGlobals::GET_FULL_HASH);
      }
    }

    if runtime_requirements.contains(RuntimeGlobals::PREFETCH_CHUNK) {
      runtime_requirements_mut.insert(RuntimeGlobals::PREFETCH_CHUNK_HANDLERS);
    }

    if runtime_requirements.contains(RuntimeGlobals::PRELOAD_CHUNK) {
      runtime_requirements_mut.insert(RuntimeGlobals::PRELOAD_CHUNK_HANDLERS);
    }

    if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK) {
      let c = compilation.chunk_by_ukey.expect_get(chunk);
      let has_async_chunks = c.has_async_chunks(&compilation.chunk_group_by_ukey);
      if has_async_chunks {
        runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
      }
      compilation
        .add_runtime_module(chunk, EnsureChunkRuntimeModule::default().boxed())
        .await?;
    }

    if runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_INCLUDE_ENTRIES) {
      runtime_requirements_mut.insert(RuntimeGlobals::ENSURE_CHUNK_HANDLERS);
    }

    let library_type = {
      let chunk = compilation.chunk_by_ukey.expect_get(chunk);
      chunk
        .get_entry_options(&compilation.chunk_group_by_ukey)
        .and_then(|options| options.library.as_ref())
        .or(compilation.options.output.library.as_ref())
        .map(|library| library.library_type.clone())
    };

    for runtime_requirement in runtime_requirements.iter() {
      match runtime_requirement {
        RuntimeGlobals::ASYNC_MODULE => {
          compilation
            .add_runtime_module(chunk, AsyncRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::BASE_URI
          if is_enabled_for_chunk(chunk, &ChunkLoading::Disable, compilation) =>
        {
          compilation
            .add_runtime_module(chunk, BaseUriRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::PUBLIC_PATH => {
          match &public_path {
            // TODO string publicPath support [hash] placeholder
            PublicPath::String(str) => {
              compilation
                .add_runtime_module(
                  chunk,
                  PublicPathRuntimeModule::new(str.as_str().into()).boxed(),
                )
                .await?;
            }
            PublicPath::Auto => {
              compilation
                .add_runtime_module(chunk, AutoPublicPathRuntimeModule::default().boxed())
                .await?;
            }
          }
        }
        RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME => {
          compilation
            .add_runtime_module(
              chunk,
              GetChunkFilenameRuntimeModule::new(
                "javascript",
                "javascript",
                SourceType::JavaScript,
                RuntimeGlobals::GET_CHUNK_SCRIPT_FILENAME.to_string(),
                |_| false,
                |chunk, compilation| {
                  Some(
                    get_js_chunk_filename_template(
                      chunk,
                      &compilation.options.output,
                      &compilation.chunk_group_by_ukey,
                    )
                    .clone(),
                  )
                },
              )
              .boxed(),
            )
            .await?;
        }
        RuntimeGlobals::GET_CHUNK_CSS_FILENAME => {
          compilation
            .add_runtime_module(
              chunk,
              GetChunkFilenameRuntimeModule::new(
                "css",
                "css",
                SourceType::Css,
                RuntimeGlobals::GET_CHUNK_CSS_FILENAME.to_string(),
                |runtime_requirements| {
                  runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS)
                },
                |chunk, compilation| {
                  chunk_has_css(&chunk.ukey, compilation).then(|| {
                    get_css_chunk_filename_template(
                      chunk,
                      &compilation.options.output,
                      &compilation.chunk_group_by_ukey,
                    )
                    .clone()
                  })
                },
              )
              .boxed(),
            )
            .await?;
        }
        RuntimeGlobals::GET_CHUNK_UPDATE_SCRIPT_FILENAME => {
          compilation
            .add_runtime_module(
              chunk,
              GetChunkUpdateFilenameRuntimeModule::default().boxed(),
            )
            .await?;
        }
        RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME => {
          compilation
            .add_runtime_module(
              chunk,
              GetMainFilenameRuntimeModule::new(
                "update manifest",
                RuntimeGlobals::GET_UPDATE_MANIFEST_FILENAME,
                compilation
                  .options
                  .output
                  .hot_update_main_filename
                  .template()
                  .to_string(),
              )
              .boxed(),
            )
            .await?;
        }
        RuntimeGlobals::LOAD_SCRIPT => {
          compilation
            .add_runtime_module(
              chunk,
              LoadScriptRuntimeModule::new(
                compilation.options.output.unique_name.clone(),
                compilation.options.output.trusted_types.is_some(),
              )
              .boxed(),
            )
            .await?;
        }
        RuntimeGlobals::HAS_OWN_PROPERTY => {
          compilation
            .add_runtime_module(chunk, HasOwnPropertyRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::GET_FULL_HASH => {
          compilation
            .add_runtime_module(chunk, GetFullHashRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::GLOBAL => {
          compilation
            .add_runtime_module(chunk, GlobalRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::CREATE_SCRIPT_URL => {
          compilation
            .add_runtime_module(chunk, CreateScriptUrlRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::ON_CHUNKS_LOADED => {
          compilation
            .add_runtime_module(chunk, OnChunkLoadedRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::DEFINE_PROPERTY_GETTERS => {
          compilation
            .add_runtime_module(chunk, DefinePropertyGettersRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::GET_TRUSTED_TYPES_POLICY => {
          compilation
            .add_runtime_module(chunk, Box::<GetTrustedTypesPolicyRuntimeModule>::default())
            .await?;
        }
        RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT => {
          compilation
            .add_runtime_module(
              chunk,
              CreateFakeNamespaceObjectRuntimeModule::default().boxed(),
            )
            .await?;
        }
        RuntimeGlobals::MAKE_NAMESPACE_OBJECT => {
          compilation
            .add_runtime_module(chunk, MakeNamespaceObjectRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT => {
          compilation
            .add_runtime_module(
              chunk,
              CompatGetDefaultExportRuntimeModule::default().boxed(),
            )
            .await?;
        }
        RuntimeGlobals::HARMONY_MODULE_DECORATOR => {
          compilation
            .add_runtime_module(
              chunk,
              HarmonyModuleDecoratorRuntimeModule::default().boxed(),
            )
            .await?;
        }
        RuntimeGlobals::NODE_MODULE_DECORATOR => {
          compilation
            .add_runtime_module(chunk, NodeModuleDecoratorRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::SYSTEM_CONTEXT if matches!(&library_type, Some(t) if t == "system") => {
          compilation
            .add_runtime_module(chunk, SystemContextRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::SCRIPT_NONCE => {
          compilation
            .add_runtime_module(chunk, NonceRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::RELATIVE_URL => {
          compilation
            .add_runtime_module(chunk, RelativeUrlRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::CHUNK_NAME => {
          compilation
            .add_runtime_module(chunk, ChunkNameRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::RUNTIME_ID => {
          compilation
            .add_runtime_module(chunk, RuntimeIdRuntimeModule::default().boxed())
            .await?;
        }
        RuntimeGlobals::PREFETCH_CHUNK => {
          compilation
            .add_runtime_module(
              chunk,
              ChunkPrefetchPreloadFunctionRuntimeModule::new(
                "prefetch",
                RuntimeGlobals::PREFETCH_CHUNK,
                RuntimeGlobals::PREFETCH_CHUNK_HANDLERS,
              )
              .boxed(),
            )
            .await?;
        }
        RuntimeGlobals::PRELOAD_CHUNK => {
          compilation
            .add_runtime_module(
              chunk,
              ChunkPrefetchPreloadFunctionRuntimeModule::new(
                "preload",
                RuntimeGlobals::PRELOAD_CHUNK,
                RuntimeGlobals::PRELOAD_CHUNK_HANDLERS,
              )
              .boxed(),
            )
            .await?;
        }
        _ => {}
      }
    }

    Ok(())
  }

  fn js_chunk_hash(
    &self,
    _ctx: PluginContext,
    args: &mut JsChunkHashArgs,
  ) -> PluginJsChunkHashHookOutput {
    for identifier in args
      .compilation
      .chunk_graph
      .get_chunk_runtime_modules_iterable(args.chunk_ukey)
    {
      if let Some((hash, _)) = args
        .compilation
        .runtime_module_code_generation_results
        .get(identifier)
      {
        hash.hash(&mut args.hasher);
      }
    }
    Ok(())
  }
}
