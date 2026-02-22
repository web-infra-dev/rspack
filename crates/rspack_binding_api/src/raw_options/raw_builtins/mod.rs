mod raw_banner;
mod raw_bundle_info;
mod raw_circular_dependency;
mod raw_context_replacement;
mod raw_copy;
mod raw_css_chunking;
mod raw_css_extract;
mod raw_dll;
mod raw_esm_lib;
mod raw_html;
mod raw_http_uri;
mod raw_ids;
mod raw_ignore;
mod raw_lazy_compilation;
mod raw_lightning_css_minimizer;
mod raw_limit_chunk_count;
mod raw_mf;
mod raw_normal_replacement;
mod raw_progress;
mod raw_runtime_chunk;
mod raw_size_limits;
mod raw_sri;
mod raw_swc_js_minimizer;

use std::cell::RefCell;

use napi::{
  Either, Env, Unknown,
  bindgen_prelude::{ClassInstance, FromNapiValue, JsObjectValue, Object},
};
use napi_derive::napi;
use raw_dll::{RawDllReferenceAgencyPluginOptions, RawFlagAllModulesAsUsedPluginOptions};
use raw_ids::RawOccurrenceChunkIdsPluginOptions;
use raw_lightning_css_minimizer::RawLightningCssMinimizerRspackPluginOptions;
use raw_mf::{
  RawCollectShareEntryPluginOptions, RawModuleFederationManifestPluginOptions,
  RawModuleFederationRuntimePluginOptions, RawProvideOptions,
  RawSharedUsedExportsOptimizerPluginOptions,
};
use raw_sri::RawSubresourceIntegrityPluginOptions;
use rspack_core::{BoxPlugin, Plugin, PluginExt};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin, NaturalChunkIdsPlugin, NaturalModuleIdsPlugin, OccurrenceChunkIdsPlugin,
};
use rspack_plugin_asset::AssetPlugin;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_case_sensitive::CaseSensitivePlugin;
use rspack_plugin_circular_dependencies::CircularDependencyRspackPlugin;
use rspack_plugin_copy::{CopyRspackPlugin, CopyRspackPluginOptions};
use rspack_plugin_css::CssPlugin;
use rspack_plugin_css_chunking::CssChunkingPlugin;
use rspack_plugin_devtool::{
  EvalDevToolModulePlugin, EvalSourceMapDevToolPlugin, SourceMapDevToolModuleOptionsPlugin,
  SourceMapDevToolModuleOptionsPluginOptions, SourceMapDevToolPlugin,
};
use rspack_plugin_dll::{
  DllEntryPlugin, DllReferenceAgencyPlugin, FlagAllModulesAsUsedPlugin, LibManifestPlugin,
};
use rspack_plugin_dynamic_entry::DynamicEntryPlugin;
use rspack_plugin_ensure_chunk_conditions::EnsureChunkConditionsPlugin;
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_esm_library::EsmLibraryPlugin;
use rspack_plugin_externals::{
  ExternalsPlugin, electron_target_plugin, http_externals_rspack_plugin, node_target_plugin,
};
use rspack_plugin_hmr::HotModuleReplacementPlugin;
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_ignore::IgnorePlugin;
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, InferAsyncModulesPlugin,
  InlineExportsPlugin, JsPlugin, MangleExportsPlugin, ModuleConcatenationPlugin,
  SideEffectsFlagPlugin, api_plugin::APIPlugin, define_plugin::DefinePlugin,
  provide_plugin::ProvidePlugin, url_plugin::URLPlugin,
};
use rspack_plugin_json::JsonPlugin;
use rspack_plugin_library::enable_library_plugin;
use rspack_plugin_lightning_css_minimizer::LightningCssMinimizerRspackPlugin;
use rspack_plugin_limit_chunk_count::LimitChunkCountPlugin;
use rspack_plugin_merge_duplicate_chunks::MergeDuplicateChunksPlugin;
use rspack_plugin_mf::{
  CollectSharedEntryPlugin, ConsumeSharedPlugin, ContainerPlugin, ContainerReferencePlugin,
  ModuleFederationManifestPlugin, ModuleFederationRuntimePlugin, ProvideSharedPlugin,
  ShareRuntimePlugin, SharedContainerPlugin, SharedUsedExportsOptimizerPlugin,
};
use rspack_plugin_module_info_header::ModuleInfoHeaderPlugin;
use rspack_plugin_module_replacement::{ContextReplacementPlugin, NormalModuleReplacementPlugin};
use rspack_plugin_no_emit_on_errors::NoEmitOnErrorsPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_remove_duplicate_modules::RemoveDuplicateModulesPlugin;
use rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin;
use rspack_plugin_rsc::{RscClientPlugin, RscServerPlugin};
use rspack_plugin_rslib::RslibPlugin;
use rspack_plugin_runtime::{
  ArrayPushCallbackChunkFormatPlugin, BundlerInfoPlugin, ChunkPrefetchPreloadPlugin,
  CommonJsChunkFormatPlugin, ModuleChunkFormatPlugin, RuntimePlugin, enable_chunk_loading_plugin,
};
use rspack_plugin_runtime_chunk::RuntimeChunkPlugin;
use rspack_plugin_schemes::{DataUriPlugin, FileUriPlugin};
use rspack_plugin_size_limits::SizeLimitsPlugin;
use rspack_plugin_sri::{SubresourceIntegrityPlugin, SubresourceIntegrityPluginOptions};
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin;
use rspack_plugin_wasm::{
  AsyncWasmPlugin, FetchCompileAsyncWasmPlugin, enable_wasm_loading_plugin,
};
use rspack_plugin_web_worker_template::web_worker_template_plugin;
use rspack_plugin_worker::WorkerPlugin;
use rustc_hash::FxHashMap as HashMap;

use self::{
  raw_banner::RawBannerPluginOptions,
  raw_bundle_info::{RawBundlerInfoModeWrapper, RawBundlerInfoPluginOptions},
  raw_circular_dependency::RawCircularDependencyRspackPluginOptions,
  raw_context_replacement::RawContextReplacementPluginOptions,
  raw_copy::RawCopyRspackPluginOptions,
  raw_css_chunking::RawCssChunkingPluginOptions,
  raw_css_extract::RawCssExtractPluginOption,
  raw_dll::{RawDllEntryPluginOptions, RawLibManifestPluginOptions},
  raw_html::RawHtmlRspackPluginOptions,
  raw_ignore::RawIgnorePluginOptions,
  raw_lazy_compilation::{JsBackend, RawLazyCompilationOption},
  raw_limit_chunk_count::RawLimitChunkCountPluginOptions,
  raw_mf::{
    RawConsumeSharedPluginOptions, RawContainerPluginOptions, RawContainerReferencePluginOptions,
    RawSharedContainerPluginOptions,
  },
  raw_normal_replacement::RawNormalModuleReplacementPluginOptions,
  raw_runtime_chunk::RawRuntimeChunkOptions,
  raw_size_limits::RawSizeLimitsPluginOptions,
  raw_swc_js_minimizer::RawSwcJsMinimizerRspackPluginOptions,
};
use crate::{
  options::entry::JsEntryPluginOptions,
  plugins::{
    JsCoordinator, JsLoaderRspackPlugin, JsLoaderRunnerGetter, JsRscClientPluginOptions,
    JsRscServerPluginOptions,
  },
  raw_options::{
    RawDynamicEntryPluginOptions, RawEvalDevToolModulePluginOptions, RawExternalItemWrapper,
    RawExternalsPluginOptions, RawHttpExternalsRspackPluginOptions, RawSplitChunksOptions,
    SourceMapDevToolPluginOptions, raw_builtins::raw_esm_lib::RawEsmLibraryPlugin,
  },
  rslib::RawRslibPluginOptions,
};

#[napi(string_enum)]
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum BuiltinPluginName {
  // webpack also have these plugins
  DefinePlugin,
  ProvidePlugin,
  BannerPlugin,
  IgnorePlugin,
  ProgressPlugin,
  EntryPlugin,
  DynamicEntryPlugin,
  ExternalsPlugin,
  NodeTargetPlugin,
  ElectronTargetPlugin,
  EnableChunkLoadingPlugin,
  EnableLibraryPlugin,
  EnableWasmLoadingPlugin,
  FetchCompileAsyncWasmPlugin,
  ChunkPrefetchPreloadPlugin,
  CommonJsChunkFormatPlugin,
  ArrayPushCallbackChunkFormatPlugin,
  ModuleChunkFormatPlugin,
  EsmLibraryPlugin,
  HotModuleReplacementPlugin,
  LimitChunkCountPlugin,
  WorkerPlugin,
  WebWorkerTemplatePlugin,
  MergeDuplicateChunksPlugin,
  SplitChunksPlugin,
  RemoveDuplicateModulesPlugin,
  ShareRuntimePlugin,
  SharedUsedExportsOptimizerPlugin,
  ContainerPlugin,
  ContainerReferencePlugin,
  ProvideSharedPlugin,
  ConsumeSharedPlugin,
  CollectSharedEntryPlugin,
  SharedContainerPlugin,
  ModuleFederationRuntimePlugin,
  ModuleFederationManifestPlugin,
  NamedModuleIdsPlugin,
  NaturalModuleIdsPlugin,
  DeterministicModuleIdsPlugin,
  NaturalChunkIdsPlugin,
  NamedChunkIdsPlugin,
  DeterministicChunkIdsPlugin,
  OccurrenceChunkIdsPlugin,
  RealContentHashPlugin,
  RemoveEmptyChunksPlugin,
  EnsureChunkConditionsPlugin,
  CaseSensitivePlugin,
  DataUriPlugin,
  FileUriPlugin,
  RuntimePlugin,
  JsonModulesPlugin,
  InferAsyncModulesPlugin,
  JavascriptModulesPlugin,
  AsyncWebAssemblyModulesPlugin,
  AssetModulesPlugin,
  SourceMapDevToolPlugin,
  EvalSourceMapDevToolPlugin,
  EvalDevToolModulePlugin,
  SideEffectsFlagPlugin,
  FlagDependencyExportsPlugin,
  FlagDependencyUsagePlugin,
  InlineExportsPlugin,
  MangleExportsPlugin,
  ModuleConcatenationPlugin,
  CssModulesPlugin,
  APIPlugin,
  RuntimeChunkPlugin,
  SizeLimitsPlugin,
  NoEmitOnErrorsPlugin,
  NormalModuleReplacementPlugin,
  ContextReplacementPlugin,
  DllEntryPlugin,
  DllReferenceAgencyPlugin,
  LibManifestPlugin,
  FlagAllModulesAsUsedPlugin,

  // rspack specific plugins
  // naming format follow XxxRspackPlugin
  HttpExternalsRspackPlugin,
  CopyRspackPlugin,
  HtmlRspackPlugin,
  SwcJsMinimizerRspackPlugin,
  LightningCssMinimizerRspackPlugin,
  BundlerInfoRspackPlugin,
  CssExtractRspackPlugin,
  SubresourceIntegrityPlugin,
  RsdoctorPlugin,
  RstestPlugin,
  RslibPlugin,
  CircularDependencyRspackPlugin,
  URLPlugin,

  // rspack js adapter plugins
  // naming format follow XxxRspackPlugin
  JsLoaderRspackPlugin,
  LazyCompilationPlugin,
  ModuleInfoHeaderPlugin,
  HttpUriPlugin,
  CssChunkingPlugin,

  // react server components
  RscServerPlugin,
  RscClientPlugin,
}

#[doc(hidden)]
pub type CustomPluginBuilder =
  for<'a> fn(env: Env, options: Unknown<'a>) -> napi::Result<BoxPlugin>;

thread_local! {
  static CUSTOMED_PLUGINS_CTOR: RefCell<HashMap<CustomPluginName, CustomPluginBuilder>> = RefCell::new(HashMap::default());
}

#[doc(hidden)]
#[allow(clippy::result_unit_err)]
pub fn register_custom_plugin(
  name: String,
  plugin_builder: CustomPluginBuilder,
) -> std::result::Result<(), ()> {
  CUSTOMED_PLUGINS_CTOR.with_borrow_mut(|ctors| match ctors.entry(name) {
    std::collections::hash_map::Entry::Occupied(_) => Err(()),
    std::collections::hash_map::Entry::Vacant(vacant_entry) => {
      vacant_entry.insert(plugin_builder);
      Ok(())
    }
  })
}

type CustomPluginName = String;

#[napi(object)]
pub struct BuiltinPlugin<'a> {
  pub name: Either<BuiltinPluginName, CustomPluginName>,
  pub options: Unknown<'a>,
  pub can_inherent_from_parent: Option<bool>,
}

impl<'a> BuiltinPlugin<'a> {
  pub fn append_to(
    self,
    env: Env,
    compiler_object: &mut Object,
    plugins: &mut Vec<BoxPlugin>,
  ) -> napi::Result<()> {
    let name = match self.name {
      Either::A(name) => name,
      Either::B(name) => {
        CUSTOMED_PLUGINS_CTOR.with_borrow(|ctors| {
          let ctor = ctors.get(&name).ok_or_else(|| {
            napi::Error::from_reason(format!("Expected plugin installed '{name}'"))
          })?;
          plugins.push(ctor(env, self.options)?);
          Ok::<_, napi::Error>(())
        })?;
        return Ok(());
      }
    };
    match name {
      // webpack also have these plugins
      BuiltinPluginName::DefinePlugin => {
        let plugin = DefinePlugin::new(
          downcast_into(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ProvidePlugin => {
        let plugin = ProvidePlugin::new(
          downcast_into(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::URLPlugin => {
        plugins.push(URLPlugin::default().boxed());
      }
      BuiltinPluginName::BannerPlugin => {
        let plugin = BannerPlugin::new(
          downcast_into::<RawBannerPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .try_into()
            .map_err(|report: rspack_error::Error| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::IgnorePlugin => {
        let plugin = IgnorePlugin::new(
          downcast_into::<RawIgnorePluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ProgressPlugin => {
        #[cfg(not(feature = "browser"))]
        {
          use rspack_plugin_progress::ProgressPlugin;

          use crate::raw_options::raw_builtins::raw_progress::RawProgressPluginOptions;

          let plugin = ProgressPlugin::new(
            downcast_into::<RawProgressPluginOptions>(self.options)
              .map_err(|report| napi::Error::from_reason(report.to_string()))?
              .into(),
          )
          .boxed();
          plugins.push(plugin);
        }
      }
      BuiltinPluginName::EntryPlugin => {
        let plugin_options = downcast_into::<JsEntryPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let context = plugin_options.context.into();
        let entry_request = plugin_options.entry;
        let options = plugin_options.options.into();
        let plugin = EntryPlugin::new(context, entry_request, options).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::DynamicEntryPlugin => {
        let plugin = DynamicEntryPlugin::new(
          downcast_into::<RawDynamicEntryPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ExternalsPlugin => {
        let plugin_options = downcast_into::<RawExternalsPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let externals = plugin_options
          .externals
          .into_iter()
          .map(|e| RawExternalItemWrapper(e).try_into())
          .collect::<Result<Vec<_>>>()
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let plugin = ExternalsPlugin::new(
          plugin_options.r#type,
          externals,
          plugin_options.place_in_initial,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::NodeTargetPlugin => plugins.push(node_target_plugin()),
      BuiltinPluginName::ElectronTargetPlugin => {
        let context = downcast_into::<String>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        electron_target_plugin(context.into(), plugins);
      }
      BuiltinPluginName::EnableChunkLoadingPlugin => {
        let chunk_loading_type = downcast_into::<String>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        enable_chunk_loading_plugin(chunk_loading_type.as_str().into(), plugins);
      }
      BuiltinPluginName::EnableLibraryPlugin => {
        let library_type = downcast_into::<String>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        enable_library_plugin(library_type, plugins);
      }
      BuiltinPluginName::EnableWasmLoadingPlugin => {
        let wasm_loading_type = downcast_into::<String>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(enable_wasm_loading_plugin(
          wasm_loading_type.as_str().into(),
        ));
      }
      BuiltinPluginName::FetchCompileAsyncWasmPlugin => {
        plugins.push(FetchCompileAsyncWasmPlugin::default().boxed());
      }
      BuiltinPluginName::ChunkPrefetchPreloadPlugin => {
        plugins.push(ChunkPrefetchPreloadPlugin::default().boxed());
      }
      BuiltinPluginName::CommonJsChunkFormatPlugin => {
        plugins.push(CommonJsChunkFormatPlugin::default().boxed());
      }
      BuiltinPluginName::EsmLibraryPlugin => {
        let options = downcast_into::<RawEsmLibraryPlugin>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(
          EsmLibraryPlugin::new(
            options.preserve_modules.as_deref().map(Into::into),
            options.split_chunks.map(Into::into),
          )
          .boxed(),
        );
      }
      BuiltinPluginName::ArrayPushCallbackChunkFormatPlugin => {
        plugins.push(ArrayPushCallbackChunkFormatPlugin::default().boxed());
      }
      BuiltinPluginName::ModuleChunkFormatPlugin => {
        plugins.push(ModuleChunkFormatPlugin::default().boxed());
      }
      BuiltinPluginName::HotModuleReplacementPlugin => {
        plugins.push(HotModuleReplacementPlugin::default().boxed());
      }
      BuiltinPluginName::LimitChunkCountPlugin => {
        let plugin = LimitChunkCountPlugin::new(
          downcast_into::<RawLimitChunkCountPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::WorkerPlugin => {
        plugins.push(WorkerPlugin::default().boxed());
      }
      BuiltinPluginName::WebWorkerTemplatePlugin => {
        web_worker_template_plugin(plugins);
      }
      BuiltinPluginName::MergeDuplicateChunksPlugin => {
        plugins.push(MergeDuplicateChunksPlugin::default().boxed());
      }
      BuiltinPluginName::SplitChunksPlugin => {
        use rspack_plugin_split_chunks::SplitChunksPlugin;
        let options = downcast_into::<RawSplitChunksOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?
          .into();
        plugins.push(SplitChunksPlugin::new(options).boxed());
      }
      BuiltinPluginName::RemoveDuplicateModulesPlugin => {
        plugins.push(RemoveDuplicateModulesPlugin::default().boxed());
      }
      BuiltinPluginName::ShareRuntimePlugin => plugins.push(
        ShareRuntimePlugin::new(
          downcast_into::<bool>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed(),
      ),
      BuiltinPluginName::SharedUsedExportsOptimizerPlugin => {
        let options = downcast_into::<RawSharedUsedExportsOptimizerPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?
          .into();
        plugins.push(SharedUsedExportsOptimizerPlugin::new(options).boxed());
      }
      BuiltinPluginName::ContainerPlugin => {
        plugins.push(
          ContainerPlugin::new(
            downcast_into::<RawContainerPluginOptions>(self.options)
              .map_err(|report| napi::Error::from_reason(report.to_string()))?
              .into(),
          )
          .boxed(),
        );
      }
      BuiltinPluginName::ContainerReferencePlugin => {
        plugins.push(
          ContainerReferencePlugin::new(
            downcast_into::<RawContainerReferencePluginOptions>(self.options)
              .map_err(|report| napi::Error::from_reason(report.to_string()))?
              .into(),
          )
          .boxed(),
        );
      }
      BuiltinPluginName::ProvideSharedPlugin => {
        let mut provides: Vec<_> = downcast_into::<Vec<RawProvideOptions>>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?
          .into_iter()
          .map(Into::into)
          .collect();
        provides.sort_unstable_by_key(|(k, _)| k.clone());
        plugins.push(ProvideSharedPlugin::new(provides).boxed())
      }
      BuiltinPluginName::CollectSharedEntryPlugin => {
        let options = downcast_into::<RawCollectShareEntryPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?
          .into();
        plugins.push(CollectSharedEntryPlugin::new(options).boxed())
      }
      BuiltinPluginName::SharedContainerPlugin => {
        let options = downcast_into::<RawSharedContainerPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?
          .into();
        plugins.push(SharedContainerPlugin::new(options).boxed())
      }
      BuiltinPluginName::ConsumeSharedPlugin => plugins.push(
        ConsumeSharedPlugin::new(
          downcast_into::<RawConsumeSharedPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed(),
      ),
      BuiltinPluginName::ModuleFederationRuntimePlugin => {
        let options = downcast_into::<RawModuleFederationRuntimePluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(ModuleFederationRuntimePlugin::new(options.into()).boxed())
      }
      BuiltinPluginName::ModuleFederationManifestPlugin => {
        let options = downcast_into::<RawModuleFederationManifestPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(ModuleFederationManifestPlugin::new(options.into()).boxed())
      }
      BuiltinPluginName::NamedModuleIdsPlugin => {
        plugins.push(NamedModuleIdsPlugin::default().boxed())
      }
      BuiltinPluginName::NaturalModuleIdsPlugin => {
        plugins.push(NaturalModuleIdsPlugin::default().boxed())
      }
      BuiltinPluginName::DeterministicModuleIdsPlugin => {
        plugins.push(DeterministicModuleIdsPlugin::default().boxed())
      }
      BuiltinPluginName::NaturalChunkIdsPlugin => {
        plugins.push(NaturalChunkIdsPlugin::default().boxed())
      }
      BuiltinPluginName::NamedChunkIdsPlugin => {
        plugins.push(NamedChunkIdsPlugin::new(None, None).boxed())
      }
      BuiltinPluginName::DeterministicChunkIdsPlugin => {
        plugins.push(DeterministicChunkIdsPlugin::default().boxed())
      }
      BuiltinPluginName::OccurrenceChunkIdsPlugin => plugins.push(
        OccurrenceChunkIdsPlugin::new(
          downcast_into::<RawOccurrenceChunkIdsPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed(),
      ),
      BuiltinPluginName::RealContentHashPlugin => {
        plugins.push(RealContentHashPlugin::default().boxed())
      }
      BuiltinPluginName::RemoveEmptyChunksPlugin => {
        plugins.push(RemoveEmptyChunksPlugin::default().boxed())
      }
      BuiltinPluginName::EnsureChunkConditionsPlugin => {
        plugins.push(EnsureChunkConditionsPlugin::default().boxed())
      }
      BuiltinPluginName::CaseSensitivePlugin => {
        plugins.push(CaseSensitivePlugin::default().boxed())
      }
      BuiltinPluginName::DataUriPlugin => plugins.push(DataUriPlugin::default().boxed()),
      BuiltinPluginName::FileUriPlugin => plugins.push(FileUriPlugin::default().boxed()),
      BuiltinPluginName::HttpUriPlugin => {
        let plugin_options =
          downcast_into::<self::raw_http_uri::RawHttpUriPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(raw_http_uri::get_http_uri_plugin(plugin_options));
      }
      BuiltinPluginName::RuntimePlugin => plugins.push(RuntimePlugin::default().boxed()),
      BuiltinPluginName::JsonModulesPlugin => plugins.push(JsonPlugin.boxed()),
      BuiltinPluginName::InferAsyncModulesPlugin => {
        plugins.push(InferAsyncModulesPlugin::default().boxed())
      }
      BuiltinPluginName::JavascriptModulesPlugin => plugins.push(JsPlugin::default().boxed()),
      BuiltinPluginName::AsyncWebAssemblyModulesPlugin => {
        plugins.push(AsyncWasmPlugin::default().boxed())
      }
      BuiltinPluginName::AssetModulesPlugin => plugins.push(AssetPlugin::default().boxed()),
      BuiltinPluginName::SourceMapDevToolPlugin => {
        let options: rspack_plugin_devtool::SourceMapDevToolPluginOptions =
          downcast_into::<SourceMapDevToolPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into();
        plugins.push(
          SourceMapDevToolModuleOptionsPlugin::new(SourceMapDevToolModuleOptionsPluginOptions {
            module: options.module,
            cheap: !options.columns,
          })
          .boxed(),
        );
        plugins.push(SourceMapDevToolPlugin::new(options).boxed());
      }
      BuiltinPluginName::EvalSourceMapDevToolPlugin => {
        let options: rspack_plugin_devtool::SourceMapDevToolPluginOptions =
          downcast_into::<SourceMapDevToolPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into();
        plugins.push(
          SourceMapDevToolModuleOptionsPlugin::new(SourceMapDevToolModuleOptionsPluginOptions {
            module: options.module,
            cheap: !options.columns,
          })
          .boxed(),
        );
        plugins.push(EvalSourceMapDevToolPlugin::new(options).boxed());
      }
      BuiltinPluginName::EvalDevToolModulePlugin => {
        plugins.push(
          EvalDevToolModulePlugin::new(
            downcast_into::<RawEvalDevToolModulePluginOptions>(self.options)
              .map_err(|report| napi::Error::from_reason(report.to_string()))?
              .into(),
          )
          .boxed(),
        );
      }
      BuiltinPluginName::SideEffectsFlagPlugin => {
        plugins.push(SideEffectsFlagPlugin::default().boxed())
      }
      BuiltinPluginName::FlagDependencyExportsPlugin => {
        plugins.push(FlagDependencyExportsPlugin::default().boxed())
      }
      BuiltinPluginName::FlagDependencyUsagePlugin => plugins.push(
        FlagDependencyUsagePlugin::new(
          downcast_into::<bool>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed(),
      ),
      BuiltinPluginName::InlineExportsPlugin => {
        plugins.push(InlineExportsPlugin::default().boxed())
      }
      BuiltinPluginName::MangleExportsPlugin => plugins.push(
        MangleExportsPlugin::new(
          downcast_into::<bool>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed(),
      ),
      BuiltinPluginName::ModuleConcatenationPlugin => {
        plugins.push(ModuleConcatenationPlugin::default().boxed())
      }
      BuiltinPluginName::CssModulesPlugin => plugins.push(CssPlugin::default().boxed()),
      BuiltinPluginName::APIPlugin => plugins.push(APIPlugin::default().boxed()),
      BuiltinPluginName::RuntimeChunkPlugin => plugins.push(
        RuntimeChunkPlugin::new(
          downcast_into::<RawRuntimeChunkOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed(),
      ),
      BuiltinPluginName::SizeLimitsPlugin => {
        let plugin = SizeLimitsPlugin::new(
          downcast_into::<RawSizeLimitsPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin)
      }

      // rspack specific plugins
      BuiltinPluginName::HttpExternalsRspackPlugin => {
        let plugin_options = downcast_into::<RawHttpExternalsRspackPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let plugin = http_externals_rspack_plugin(plugin_options.css, plugin_options.web_async);
        plugins.push(plugin);
      }
      BuiltinPluginName::SwcJsMinimizerRspackPlugin => {
        let plugin = SwcJsMinimizerRspackPlugin::new(
          downcast_into::<RawSwcJsMinimizerRspackPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .try_into()
            .map_err(|report: rspack_error::Error| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::LightningCssMinimizerRspackPlugin => plugins.push(
        LightningCssMinimizerRspackPlugin::new(
          downcast_into::<RawLightningCssMinimizerRspackPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .try_into()
            .map_err(|report: rspack_error::Error| napi::Error::from_reason(report.to_string()))?,
        )
        .boxed(),
      ),
      BuiltinPluginName::CopyRspackPlugin => {
        let plugin = CopyRspackPlugin::new(
          CopyRspackPluginOptions::from(
            downcast_into::<RawCopyRspackPluginOptions>(self.options)
              .map_err(|report| napi::Error::from_reason(report.to_string()))?,
          )
          .patterns,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::HtmlRspackPlugin => {
        let plugin = HtmlRspackPlugin::new(
          downcast_into::<RawHtmlRspackPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::BundlerInfoRspackPlugin => {
        let plugin_options = downcast_into::<RawBundlerInfoPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(
          BundlerInfoPlugin::new(
            plugin_options.version,
            plugin_options.bundler,
            RawBundlerInfoModeWrapper(plugin_options.force).into(),
          )
          .boxed(),
        )
      }
      BuiltinPluginName::CssExtractRspackPlugin => {
        let plugin = rspack_plugin_extract_css::plugin::PluginCssExtract::new(
          downcast_into::<RawCssExtractPluginOption>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::CircularDependencyRspackPlugin => plugins.push(
        CircularDependencyRspackPlugin::new(
          downcast_into::<RawCircularDependencyRspackPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed(),
      ),
      BuiltinPluginName::JsLoaderRspackPlugin => {
        // Set the compiler._runLoader property on the JsObject to ensure that the runLoader
        // is not garbage collected by JS while the stats Object holds a reference to JsLoaderPlugin.
        compiler_object.set_named_property("_runLoader", self.options)?;
        let loader_runner_getter = JsLoaderRunnerGetter::new(&env)?;
        plugins.push(JsLoaderRspackPlugin::new(loader_runner_getter).boxed());
      }
      BuiltinPluginName::LazyCompilationPlugin => {
        let options = downcast_into::<RawLazyCompilationOption>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let js_backend = JsBackend::from(&options);
        plugins.push(
          Box::new(rspack_plugin_lazy_compilation::LazyCompilationPlugin::new(
            js_backend,
            options.test.map(|test| test.into()),
            options.entries,
            options.imports,
            options.client,
          )) as Box<dyn Plugin>,
        )
      }
      BuiltinPluginName::NoEmitOnErrorsPlugin => {
        plugins.push(NoEmitOnErrorsPlugin::default().boxed());
      }
      BuiltinPluginName::NormalModuleReplacementPlugin => {
        let raw_options = downcast_into::<RawNormalModuleReplacementPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(NormalModuleReplacementPlugin::new(raw_options.into()).boxed());
      }
      BuiltinPluginName::ContextReplacementPlugin => {
        let raw_options = downcast_into::<RawContextReplacementPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options
          .try_into()
          .map_err(|report: rspack_error::Error| napi::Error::from_reason(report.to_string()))?;
        plugins.push(ContextReplacementPlugin::new(options).boxed());
      }
      BuiltinPluginName::DllEntryPlugin => {
        let raw_options = downcast_into::<RawDllEntryPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(DllEntryPlugin::new(options).boxed());
      }
      BuiltinPluginName::LibManifestPlugin => {
        let raw_options = downcast_into::<RawLibManifestPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(LibManifestPlugin::new(options).boxed());
      }
      BuiltinPluginName::FlagAllModulesAsUsedPlugin => {
        let raw_options = downcast_into::<RawFlagAllModulesAsUsedPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(FlagAllModulesAsUsedPlugin::new(raw_options.explanation).boxed())
      }
      BuiltinPluginName::DllReferenceAgencyPlugin => {
        let raw_options = downcast_into::<RawDllReferenceAgencyPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(DllReferenceAgencyPlugin::new(options).boxed());
      }
      BuiltinPluginName::RsdoctorPlugin => {
        #[cfg(not(feature = "browser"))]
        {
          use rspack_plugin_rsdoctor::RsdoctorPlugin;

          use crate::rsdoctor::RawRsdoctorPluginOptions;

          let raw_options = downcast_into::<RawRsdoctorPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?;
          let options = raw_options.into();
          plugins.push(RsdoctorPlugin::new(options).boxed());
        }
      }
      BuiltinPluginName::RstestPlugin => {
        #[cfg(not(feature = "browser"))]
        {
          use rspack_plugin_rstest::RstestPlugin;

          use crate::rstest::RawRstestPluginOptions;

          let raw_options = downcast_into::<RawRstestPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?;
          let options = raw_options.into();
          plugins.push(RstestPlugin::new(options).boxed());
        }
      }
      BuiltinPluginName::RslibPlugin => {
        let raw_options = downcast_into::<RawRslibPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(RslibPlugin::new(options).boxed());
      }
      BuiltinPluginName::SubresourceIntegrityPlugin => {
        let raw_options = downcast_into::<RawSubresourceIntegrityPluginOptions>(self.options)
          .and_then(SubresourceIntegrityPluginOptions::try_from);
        match raw_options {
          Ok(options) => {
            plugins.push(SubresourceIntegrityPlugin::new(options, None).boxed());
          }
          Err(error) => {
            plugins.push(SubresourceIntegrityPlugin::new(Default::default(), Some(error)).boxed());
          }
        }
      }
      BuiltinPluginName::ModuleInfoHeaderPlugin => {
        let verbose = downcast_into::<bool>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(ModuleInfoHeaderPlugin::new(verbose).boxed());
      }
      BuiltinPluginName::CssChunkingPlugin => {
        let options = downcast_into::<RawCssChunkingPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(CssChunkingPlugin::new(options.into()).boxed());
      }
      BuiltinPluginName::RscServerPlugin => {
        let options = &downcast_into::<JsRscServerPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(RscServerPlugin::new(options.try_into()?).boxed());
      }
      BuiltinPluginName::RscClientPlugin => {
        let options = &downcast_into::<JsRscClientPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(RscClientPlugin::new(options.into()).boxed());
      }
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: Unknown) -> Result<T> {
  rspack_napi::downcast_into(o).to_rspack_result()
}
