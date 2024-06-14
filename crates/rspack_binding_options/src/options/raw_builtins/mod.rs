mod raw_banner;
mod raw_bundle_info;
mod raw_copy;
mod raw_css_extract;
mod raw_html;
mod raw_ignore;
mod raw_lazy_compilation;
mod raw_limit_chunk_count;
mod raw_mf;
mod raw_progress;
mod raw_runtime_chunk;
mod raw_size_limits;
mod raw_swc_js_minimizer;
mod raw_to_be_deprecated;

use napi::{bindgen_prelude::FromNapiValue, Env, JsUnknown};
use napi_derive::napi;
use rspack_core::{BoxPlugin, Define, DefinePlugin, Plugin, PluginExt, Provide, ProvidePlugin};
use rspack_error::Result;
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin, NaturalChunkIdsPlugin, NaturalModuleIdsPlugin,
};
use rspack_napi::NapiResultExt;
use rspack_plugin_asset::AssetPlugin;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::{CopyRspackPlugin, CopyRspackPluginOptions};
use rspack_plugin_css::CssPlugin;
use rspack_plugin_devtool::{
  EvalDevToolModulePlugin, EvalSourceMapDevToolPlugin, SourceMapDevToolModuleOptionsPlugin,
  SourceMapDevToolModuleOptionsPluginOptions, SourceMapDevToolPlugin,
  SourceMapDevToolPluginOptions,
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
  api_plugin::APIPlugin, FlagDependencyExportsPlugin, FlagDependencyUsagePlugin,
  InferAsyncModulesPlugin, JsPlugin, MangleExportsPlugin, ModuleConcatenationPlugin,
  SideEffectsFlagPlugin,
};
use rspack_plugin_json::JsonPlugin;
use rspack_plugin_library::enable_library_plugin;
use rspack_plugin_limit_chunk_count::LimitChunkCountPlugin;
use rspack_plugin_merge_duplicate_chunks::MergeDuplicateChunksPlugin;
use rspack_plugin_mf::{
  ConsumeSharedPlugin, ContainerPlugin, ContainerReferencePlugin, ModuleFederationRuntimePlugin,
  ProvideSharedPlugin, ShareRuntimePlugin,
};
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin;
use rspack_plugin_remove_parent_modules::RemoveParentModulesPlugin;
use rspack_plugin_runtime::{
  enable_chunk_loading_plugin, ArrayPushCallbackChunkFormatPlugin, BundlerInfoPlugin,
  ChunkPrefetchPreloadPlugin, CommonJsChunkFormatPlugin, ModuleChunkFormatPlugin, RuntimePlugin,
};
use rspack_plugin_runtime_chunk::RuntimeChunkPlugin;
use rspack_plugin_schemes::{DataUriPlugin, FileUriPlugin};
use rspack_plugin_size_limits::SizeLimitsPlugin;
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerRspackPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin;
use rspack_plugin_warn_sensitive_module::WarnCaseSensitiveModulesPlugin;
use rspack_plugin_wasm::{enable_wasm_loading_plugin, AsyncWasmPlugin};
use rspack_plugin_web_worker_template::web_worker_template_plugin;
use rspack_plugin_worker::WorkerPlugin;

pub use self::{
  raw_banner::RawBannerPluginOptions, raw_copy::RawCopyRspackPluginOptions,
  raw_html::RawHtmlRspackPluginOptions, raw_ignore::RawIgnorePluginOptions,
  raw_limit_chunk_count::RawLimitChunkCountPluginOptions, raw_mf::RawContainerPluginOptions,
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
  plugins::{CssExtractRspackAdditionalDataPlugin, JsLoaderRspackPlugin},
  JsLoaderRunner, RawDynamicEntryPluginOptions, RawEntryPluginOptions,
  RawEvalDevToolModulePluginOptions, RawExternalItemWrapper, RawExternalsPluginOptions,
  RawHttpExternalsRspackPluginOptions, RawSourceMapDevToolPluginOptions, RawSplitChunksOptions,
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

  // rspack specific plugins
  // naming format follow XxxRspackPlugin
  HttpExternalsRspackPlugin,
  CopyRspackPlugin,
  HtmlRspackPlugin,
  SwcJsMinimizerRspackPlugin,
  SwcCssMinimizerRspackPlugin,
  BundlerInfoRspackPlugin,
  CssExtractRspackPlugin,

  // rspack js adapter plugins
  // naming format follow XxxRspackPlugin
  JsLoaderRspackPlugin,
  LazyCompilationPlugin,

  RemoveParentModulesPlugin,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub name: BuiltinPluginName,
  pub options: JsUnknown,
  pub can_inherent_from_parent: Option<bool>,
}

impl BuiltinPlugin {
  pub fn append_to(self, env: Env, plugins: &mut Vec<BoxPlugin>) -> rspack_error::Result<()> {
    match self.name {
      // webpack also have these plugins
      BuiltinPluginName::DefinePlugin => {
        let plugin = DefinePlugin::new(downcast_into::<Define>(self.options)?).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ProvidePlugin => {
        let plugin = ProvidePlugin::new(downcast_into::<Provide>(self.options)?).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::BannerPlugin => {
        let plugin =
          BannerPlugin::new(downcast_into::<RawBannerPluginOptions>(self.options)?.try_into()?)
            .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::IgnorePlugin => {
        let plugin =
          IgnorePlugin::new(downcast_into::<RawIgnorePluginOptions>(self.options)?.into()).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ProgressPlugin => {
        let plugin =
          ProgressPlugin::new(downcast_into::<RawProgressPluginOptions>(self.options)?.into())
            .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::EntryPlugin => {
        let plugin_options = downcast_into::<RawEntryPluginOptions>(self.options)?;
        let context = plugin_options.context.into();
        let entry_request = plugin_options.entry;
        let options = plugin_options.options.into();
        let plugin = EntryPlugin::new(context, entry_request, options).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::DynamicEntryPlugin => {
        let plugin = DynamicEntryPlugin::new(
          downcast_into::<RawDynamicEntryPluginOptions>(self.options)?.into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::ExternalsPlugin => {
        let plugin_options = downcast_into::<RawExternalsPluginOptions>(self.options)?;
        let externals = plugin_options
          .externals
          .into_iter()
          .map(|e| RawExternalItemWrapper(e).try_into())
          .collect::<Result<Vec<_>>>()?;
        let plugin = ExternalsPlugin::new(plugin_options.r#type, externals).boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::NodeTargetPlugin => plugins.push(node_target_plugin()),
      BuiltinPluginName::ElectronTargetPlugin => {
        let context = downcast_into::<String>(self.options)?;
        electron_target_plugin(context.into(), plugins);
      }
      BuiltinPluginName::EnableChunkLoadingPlugin => {
        let chunk_loading_type = downcast_into::<String>(self.options)?;
        enable_chunk_loading_plugin(chunk_loading_type.as_str().into(), plugins);
      }
      BuiltinPluginName::EnableLibraryPlugin => {
        let library_type = downcast_into::<String>(self.options)?;
        enable_library_plugin(library_type, plugins);
      }
      BuiltinPluginName::EnableWasmLoadingPlugin => {
        let wasm_loading_type = downcast_into::<String>(self.options)?;
        plugins.push(enable_wasm_loading_plugin(
          wasm_loading_type.as_str().into(),
        ));
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
          downcast_into::<RawLimitChunkCountPluginOptions>(self.options)?.into(),
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
        let options = downcast_into::<RawSplitChunksOptions>(self.options)?.into();
        plugins.push(SplitChunksPlugin::new(options).boxed());
      }
      BuiltinPluginName::ShareRuntimePlugin => {
        plugins.push(ShareRuntimePlugin::new(downcast_into::<bool>(self.options)?).boxed())
      }
      BuiltinPluginName::ContainerPlugin => {
        plugins.push(
          ContainerPlugin::new(downcast_into::<RawContainerPluginOptions>(self.options)?.into())
            .boxed(),
        );
      }
      BuiltinPluginName::ContainerReferencePlugin => {
        plugins.push(
          ContainerReferencePlugin::new(
            downcast_into::<RawContainerReferencePluginOptions>(self.options)?.into(),
          )
          .boxed(),
        );
      }
      BuiltinPluginName::ProvideSharedPlugin => {
        let mut provides: Vec<_> = downcast_into::<Vec<RawProvideOptions>>(self.options)?
          .into_iter()
          .map(Into::into)
          .collect();
        provides.sort_unstable_by_key(|(k, _)| k.to_string());
        plugins.push(ProvideSharedPlugin::new(provides).boxed())
      }
      BuiltinPluginName::ConsumeSharedPlugin => plugins.push(
        ConsumeSharedPlugin::new(
          downcast_into::<RawConsumeSharedPluginOptions>(self.options)?.into(),
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
        let options: SourceMapDevToolPluginOptions =
          downcast_into::<RawSourceMapDevToolPluginOptions>(self.options)?.into();
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
        let options: SourceMapDevToolPluginOptions =
          downcast_into::<RawSourceMapDevToolPluginOptions>(self.options)?.into();
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
            downcast_into::<RawEvalDevToolModulePluginOptions>(self.options)?.into(),
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
      BuiltinPluginName::FlagDependencyUsagePlugin => {
        plugins.push(FlagDependencyUsagePlugin::new(downcast_into::<bool>(self.options)?).boxed())
      }
      BuiltinPluginName::MangleExportsPlugin => {
        plugins.push(MangleExportsPlugin::new(downcast_into::<bool>(self.options)?).boxed())
      }
      BuiltinPluginName::ModuleConcatenationPlugin => {
        plugins.push(ModuleConcatenationPlugin::default().boxed())
      }
      BuiltinPluginName::CssModulesPlugin => plugins.push(CssPlugin::default().boxed()),
      BuiltinPluginName::APIPlugin => plugins.push(APIPlugin::default().boxed()),
      BuiltinPluginName::RuntimeChunkPlugin => plugins.push(
        RuntimeChunkPlugin::new(downcast_into::<RawRuntimeChunkOptions>(self.options)?.into())
          .boxed(),
      ),
      BuiltinPluginName::SizeLimitsPlugin => {
        let plugin =
          SizeLimitsPlugin::new(downcast_into::<RawSizeLimitsPluginOptions>(self.options)?.into())
            .boxed();
        plugins.push(plugin)
      }

      // rspack specific plugins
      BuiltinPluginName::HttpExternalsRspackPlugin => {
        let plugin_options = downcast_into::<RawHttpExternalsRspackPluginOptions>(self.options)?;
        let plugin = http_externals_rspack_plugin(plugin_options.css, plugin_options.web_async);
        plugins.push(plugin);
      }
      BuiltinPluginName::SwcJsMinimizerRspackPlugin => {
        let plugin = SwcJsMinimizerRspackPlugin::new(
          downcast_into::<RawSwcJsMinimizerRspackPluginOptions>(self.options)?.try_into()?,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::SwcCssMinimizerRspackPlugin => {
        plugins.push(SwcCssMinimizerRspackPlugin::default().boxed())
      }
      BuiltinPluginName::CopyRspackPlugin => {
        let plugin = CopyRspackPlugin::new(
          CopyRspackPluginOptions::from(downcast_into::<RawCopyRspackPluginOptions>(self.options)?)
            .patterns,
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::HtmlRspackPlugin => {
        let plugin =
          HtmlRspackPlugin::new(downcast_into::<RawHtmlRspackPluginOptions>(self.options)?.into())
            .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::BundlerInfoRspackPlugin => {
        let plugin_options = downcast_into::<RawBundlerInfoPluginOptions>(self.options)?;
        plugins.push(
          BundlerInfoPlugin::new(
            RawBundlerInfoModeWrapper(plugin_options.force).into(),
            plugin_options.version,
          )
          .boxed(),
        )
      }
      BuiltinPluginName::CssExtractRspackPlugin => {
        let additional_data_plugin = CssExtractRspackAdditionalDataPlugin::new(env)?.boxed();
        plugins.push(additional_data_plugin);
        let plugin = rspack_plugin_extract_css::plugin::PluginCssExtract::new(
          downcast_into::<RawCssExtractPluginOption>(self.options)?.into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::JsLoaderRspackPlugin => {
        plugins
          .push(JsLoaderRspackPlugin::new(downcast_into::<JsLoaderRunner>(self.options)?).boxed());
      }
      BuiltinPluginName::LazyCompilationPlugin => {
        let options = downcast_into::<RawLazyCompilationOption>(self.options)?;
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
      BuiltinPluginName::RemoveParentModulesPlugin => {
        plugins.push(RemoveParentModulesPlugin::default().boxed());
      }
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  rspack_napi::downcast_into(o).into_rspack_result()
}

// TO BE DEPRECATED
pub use raw_to_be_deprecated::RawBuiltins;
