mod css_chunking;
mod raw_banner;
mod raw_bundle_info;
mod raw_circular_dependency;
mod raw_copy;
mod raw_css_extract;
mod raw_dll;
mod raw_html;
mod raw_http_uri;
mod raw_ids;
mod raw_ignore;
mod raw_lazy_compilation;
mod raw_lightning_css_minimizer;
mod raw_limit_chunk_count;
mod raw_mf;
mod raw_progress;
mod raw_runtime_chunk;
mod raw_size_limits;
mod raw_sri;
mod raw_swc_js_minimizer;

use napi::{
  bindgen_prelude::{FromNapiValue, Object},
  Env, JsUnknown,
};
use napi_derive::napi;
use raw_dll::{RawDllReferenceAgencyPluginOptions, RawFlagAllModulesAsUsedPluginOptions};
use raw_ids::RawOccurrenceChunkIdsPluginOptions;
use raw_lightning_css_minimizer::RawLightningCssMinimizerRspackPluginOptions;
use raw_sri::RawSubresourceIntegrityPluginOptions;
use rspack_core::{BoxPlugin, Plugin, PluginExt};
use rspack_error::{Result, ToStringResultToRspackResultExt};
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin, NaturalChunkIdsPlugin, NaturalModuleIdsPlugin, OccurrenceChunkIdsPlugin,
};
use rspack_plugin_asset::AssetPlugin;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_circular_dependencies::CircularDependencyRspackPlugin;
use rspack_plugin_context_replacement::ContextReplacementPlugin;
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
use rspack_plugin_externals::{
  electron_target_plugin, http_externals_rspack_plugin, node_target_plugin, ExternalsPlugin,
};
use rspack_plugin_hmr::HotModuleReplacementPlugin;
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_ignore::IgnorePlugin;
use rspack_plugin_javascript::{
  api_plugin::APIPlugin, define_plugin::DefinePlugin, provide_plugin::ProvidePlugin,
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, InferAsyncModulesPlugin, JsPlugin,
  MangleExportsPlugin, ModuleConcatenationPlugin, SideEffectsFlagPlugin,
};
use rspack_plugin_json::JsonPlugin;
use rspack_plugin_library::enable_library_plugin;
use rspack_plugin_lightning_css_minimizer::LightningCssMinimizerRspackPlugin;
use rspack_plugin_limit_chunk_count::LimitChunkCountPlugin;
use rspack_plugin_merge_duplicate_chunks::MergeDuplicateChunksPlugin;
use rspack_plugin_mf::{
  ConsumeSharedPlugin, ContainerPlugin, ContainerReferencePlugin, ModuleFederationRuntimePlugin,
  ProvideSharedPlugin, ShareRuntimePlugin,
};
use rspack_plugin_module_info_header::ModuleInfoHeaderPlugin;
use rspack_plugin_no_emit_on_errors::NoEmitOnErrorsPlugin;
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_remove_duplicate_modules::RemoveDuplicateModulesPlugin;
use rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin;
use rspack_plugin_rsdoctor::RsdoctorPlugin;
use rspack_plugin_rstest::RstestPlugin;
use rspack_plugin_runtime::{
  enable_chunk_loading_plugin, ArrayPushCallbackChunkFormatPlugin, BundlerInfoPlugin,
  ChunkPrefetchPreloadPlugin, CommonJsChunkFormatPlugin, ModuleChunkFormatPlugin, RuntimePlugin,
};
use rspack_plugin_runtime_chunk::RuntimeChunkPlugin;
use rspack_plugin_schemes::{DataUriPlugin, FileUriPlugin};
use rspack_plugin_size_limits::SizeLimitsPlugin;
use rspack_plugin_sri::SubresourceIntegrityPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin;
use rspack_plugin_warn_sensitive_module::WarnCaseSensitiveModulesPlugin;
use rspack_plugin_wasm::{
  enable_wasm_loading_plugin, AsyncWasmPlugin, FetchCompileAsyncWasmPlugin,
};
use rspack_plugin_web_worker_template::web_worker_template_plugin;
use rspack_plugin_worker::WorkerPlugin;

pub use self::{
  css_chunking::CssChunkingPluginOptions,
  raw_banner::RawBannerPluginOptions,
  raw_circular_dependency::RawCircularDependencyRspackPluginOptions,
  raw_copy::RawCopyRspackPluginOptions,
  raw_dll::{RawDllEntryPluginOptions, RawLibManifestPluginOptions},
  raw_html::RawHtmlRspackPluginOptions,
  raw_ignore::RawIgnorePluginOptions,
  raw_limit_chunk_count::RawLimitChunkCountPluginOptions,
  raw_mf::RawContainerPluginOptions,
  raw_progress::RawProgressPluginOptions,
  raw_swc_js_minimizer::RawSwcJsMinimizerRspackPluginOptions,
};
use self::{
  raw_bundle_info::{RawBundlerInfoModeWrapper, RawBundlerInfoPluginOptions},
  raw_css_extract::RawCssExtractPluginOption,
  raw_lazy_compilation::{JsBackend, RawLazyCompilationOption},
  raw_mf::{RawConsumeSharedPluginOptions, RawContainerReferencePluginOptions, RawProvideOptions},
  raw_runtime_chunk::RawRuntimeChunkOptions,
  raw_size_limits::RawSizeLimitsPluginOptions,
};
use crate::{
  entry::JsEntryPluginOptions, plugins::JsLoaderRspackPlugin, JsLoaderRunnerGetter,
  RawContextReplacementPluginOptions, RawDynamicEntryPluginOptions,
  RawEvalDevToolModulePluginOptions, RawExternalItemWrapper, RawExternalsPluginOptions,
  RawHttpExternalsRspackPluginOptions, RawRsdoctorPluginOptions, RawRstestPluginOptions,
  RawSplitChunksOptions, SourceMapDevToolPluginOptions,
};

#[napi(string_enum)]
#[derive(Debug)]
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
  HotModuleReplacementPlugin,
  LimitChunkCountPlugin,
  WorkerPlugin,
  WebWorkerTemplatePlugin,
  MergeDuplicateChunksPlugin,
  SplitChunksPlugin,
  RemoveDuplicateModulesPlugin,
  ShareRuntimePlugin,
  ContainerPlugin,
  ContainerReferencePlugin,
  ProvideSharedPlugin,
  ConsumeSharedPlugin,
  ModuleFederationRuntimePlugin,
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
  WarnCaseSensitiveModulesPlugin,
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
  MangleExportsPlugin,
  ModuleConcatenationPlugin,
  CssModulesPlugin,
  APIPlugin,
  RuntimeChunkPlugin,
  SizeLimitsPlugin,
  NoEmitOnErrorsPlugin,
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
  CircularDependencyRspackPlugin,

  // rspack js adapter plugins
  // naming format follow XxxRspackPlugin
  JsLoaderRspackPlugin,
  LazyCompilationPlugin,
  ModuleInfoHeaderPlugin,
  HttpUriPlugin,
  CssChunkingPlugin,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub name: BuiltinPluginName,
  pub options: JsUnknown,
  pub can_inherent_from_parent: Option<bool>,
}

impl BuiltinPlugin {
  pub fn append_to(
    self,
    env: Env,
    compiler_object: &mut Object,
    plugins: &mut Vec<BoxPlugin>,
  ) -> napi::Result<()> {
    match self.name {
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
      BuiltinPluginName::BannerPlugin => {
        let plugin = BannerPlugin::new(
          downcast_into::<RawBannerPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .try_into()
            .map_err(|report: rspack_error::miette::Error| {
              napi::Error::from_reason(report.to_string())
            })?,
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
        let plugin = ProgressPlugin::new(
          downcast_into::<RawProgressPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .into(),
        )
        .boxed();
        plugins.push(plugin);
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
        let plugin = ExternalsPlugin::new(plugin_options.r#type, externals).boxed();
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
        provides.sort_unstable_by_key(|(k, _)| k.to_string());
        plugins.push(ProvideSharedPlugin::new(provides).boxed())
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
        plugins.push(ModuleFederationRuntimePlugin::default().boxed())
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
      BuiltinPluginName::WarnCaseSensitiveModulesPlugin => {
        plugins.push(WarnCaseSensitiveModulesPlugin::default().boxed())
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
            .map_err(|report: rspack_error::miette::Error| {
              napi::Error::from_reason(report.to_string())
            })?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::LightningCssMinimizerRspackPlugin => plugins.push(
        LightningCssMinimizerRspackPlugin::new(
          downcast_into::<RawLightningCssMinimizerRspackPluginOptions>(self.options)
            .map_err(|report| napi::Error::from_reason(report.to_string()))?
            .try_into()
            .map_err(|report: rspack_error::miette::Error| {
              napi::Error::from_reason(report.to_string())
            })?,
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
        plugins.push(Box::new(
          rspack_plugin_lazy_compilation::plugin::LazyCompilationPlugin::new(
            options.cacheable,
            js_backend,
            options.test.map(|test| test.into()),
            options.entries,
            options.imports,
          ),
        ) as Box<dyn Plugin>)
      }
      BuiltinPluginName::NoEmitOnErrorsPlugin => {
        plugins.push(NoEmitOnErrorsPlugin::default().boxed());
      }
      BuiltinPluginName::ContextReplacementPlugin => {
        let raw_options = downcast_into::<RawContextReplacementPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options
          .try_into()
          .map_err(|report: rspack_error::miette::Error| {
            napi::Error::from_reason(report.to_string())
          })?;
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
        let raw_options = downcast_into::<RawRsdoctorPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(RsdoctorPlugin::new(options).boxed());
      }
      BuiltinPluginName::RstestPlugin => {
        let raw_options = downcast_into::<RawRstestPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(RstestPlugin::new(options).boxed());
      }
      BuiltinPluginName::SubresourceIntegrityPlugin => {
        let raw_options = downcast_into::<RawSubresourceIntegrityPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        let options = raw_options.into();
        plugins.push(SubresourceIntegrityPlugin::new(options).boxed());
      }
      BuiltinPluginName::ModuleInfoHeaderPlugin => {
        let verbose = downcast_into::<bool>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(ModuleInfoHeaderPlugin::new(verbose).boxed());
      }
      BuiltinPluginName::CssChunkingPlugin => {
        let options = downcast_into::<CssChunkingPluginOptions>(self.options)
          .map_err(|report| napi::Error::from_reason(report.to_string()))?;
        plugins.push(CssChunkingPlugin::new(options.into()).boxed());
      }
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  rspack_napi::downcast_into(o).to_rspack_result()
}
