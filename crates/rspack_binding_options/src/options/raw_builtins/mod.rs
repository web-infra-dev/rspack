mod raw_banner;
mod raw_bundle_info;
mod raw_copy;
mod raw_css_extract;
mod raw_html;
mod raw_limit_chunk_count;
mod raw_mf;
mod raw_progress;
mod raw_swc_js_minimizer;
mod raw_to_be_deprecated;

use napi::{bindgen_prelude::FromNapiValue, JsUnknown};
use napi_derive::napi;
use rspack_core::{BoxPlugin, Define, DefinePlugin, PluginExt, Provide, ProvidePlugin};
use rspack_error::Result;
use rspack_ids::{
  DeterministicChunkIdsPlugin, DeterministicModuleIdsPlugin, NamedChunkIdsPlugin,
  NamedModuleIdsPlugin,
};
use rspack_napi_shared::NapiResultExt;
use rspack_plugin_asset::AssetPlugin;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::{CopyRspackPlugin, CopyRspackPluginOptions};
use rspack_plugin_devtool::{
  EvalDevToolModulePlugin, EvalSourceMapDevToolPlugin, SourceMapDevToolModuleOptionsPlugin,
  SourceMapDevToolModuleOptionsPluginOptions, SourceMapDevToolPlugin,
  SourceMapDevToolPluginOptions,
};
use rspack_plugin_ensure_chunk_conditions::EnsureChunkConditionsPlugin;
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_externals::{
  electron_target_plugin, http_externals_rspack_plugin, node_target_plugin, ExternalsPlugin,
};
use rspack_plugin_hmr::HotModuleReplacementPlugin;
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_javascript::{
  FlagDependencyExportsPlugin, FlagDependencyUsagePlugin, InferAsyncModulesPlugin, JsPlugin,
  MangleExportsPlugin, ModuleConcatenationPlugin, SideEffectsFlagPlugin,
};
use rspack_plugin_json::JsonPlugin;
use rspack_plugin_library::enable_library_plugin;
use rspack_plugin_limit_chunk_count::LimitChunkCountPlugin;
use rspack_plugin_merge_duplicate_chunks::MergeDuplicateChunksPlugin;
use rspack_plugin_mf::{
  ConsumeSharedPlugin, ContainerPlugin, ContainerReferencePlugin, ProvideSharedPlugin,
  ShareRuntimePlugin,
};
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_real_content_hash::RealContentHashPlugin;
use rspack_plugin_remove_empty_chunks::RemoveEmptyChunksPlugin;
use rspack_plugin_runtime::{
  enable_chunk_loading_plugin, ArrayPushCallbackChunkFormatPlugin, BundlerInfoPlugin,
  ChunkPrefetchPreloadPlugin, CommonJsChunkFormatPlugin, ModuleChunkFormatPlugin, RuntimePlugin,
};
use rspack_plugin_schemes::{DataUriPlugin, FileUriPlugin};
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerRspackPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin;
use rspack_plugin_warn_sensitive_module::WarnCaseSensitiveModulesPlugin;
use rspack_plugin_wasm::{enable_wasm_loading_plugin, AsyncWasmPlugin};
use rspack_plugin_web_worker_template::web_worker_template_plugin;
use rspack_plugin_worker::WorkerPlugin;

pub use self::{
  raw_banner::RawBannerPluginOptions, raw_copy::RawCopyRspackPluginOptions,
  raw_html::RawHtmlRspackPluginOptions, raw_limit_chunk_count::RawLimitChunkCountPluginOptions,
  raw_mf::RawContainerPluginOptions, raw_progress::RawProgressPluginOptions,
  raw_swc_js_minimizer::RawSwcJsMinimizerRspackPluginOptions,
};
use self::{
  raw_bundle_info::{RawBundlerInfoModeWrapper, RawBundlerInfoPluginOptions},
  raw_css_extract::RawCssExtractPluginOption,
  raw_mf::{RawConsumeSharedPluginOptions, RawContainerReferencePluginOptions, RawProvideOptions},
};
use crate::{
  RawEntryPluginOptions, RawEvalDevToolModulePluginOptions, RawExternalItemWrapper,
  RawExternalsPluginOptions, RawHttpExternalsRspackPluginOptions, RawSourceMapDevToolPluginOptions,
  RawSplitChunksOptions,
};

#[napi(string_enum)]
#[derive(Debug)]
pub enum BuiltinPluginName {
  // webpack also have these plugins
  DefinePlugin,
  ProvidePlugin,
  BannerPlugin,
  ProgressPlugin,
  EntryPlugin,
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
  NamedModuleIdsPlugin,
  DeterministicModuleIdsPlugin,
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

  // rspack specific plugins
  // naming format follow XxxRspackPlugin
  HttpExternalsRspackPlugin,
  CopyRspackPlugin,
  HtmlRspackPlugin,
  SwcJsMinimizerRspackPlugin,
  SwcCssMinimizerRspackPlugin,
  BundlerInfoRspackPlugin,
  CssExtractPlugin,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub name: BuiltinPluginName,
  pub options: JsUnknown,
  pub can_inherent_from_parent: Option<bool>,
}

impl BuiltinPlugin {
  pub fn append_to(self, plugins: &mut Vec<BoxPlugin>) -> rspack_error::Result<()> {
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
        plugins.push(ChunkPrefetchPreloadPlugin.boxed());
      }
      BuiltinPluginName::CommonJsChunkFormatPlugin => {
        plugins.push(CommonJsChunkFormatPlugin.boxed());
      }
      BuiltinPluginName::ArrayPushCallbackChunkFormatPlugin => {
        plugins.push(ArrayPushCallbackChunkFormatPlugin.boxed());
      }
      BuiltinPluginName::ModuleChunkFormatPlugin => {
        plugins.push(ModuleChunkFormatPlugin.boxed());
      }
      BuiltinPluginName::HotModuleReplacementPlugin => {
        plugins.push(HotModuleReplacementPlugin.boxed());
      }
      BuiltinPluginName::LimitChunkCountPlugin => {
        let plugin = LimitChunkCountPlugin::new(
          downcast_into::<RawLimitChunkCountPluginOptions>(self.options)?.into(),
        )
        .boxed();
        plugins.push(plugin);
      }
      BuiltinPluginName::WorkerPlugin => {
        plugins.push(WorkerPlugin.boxed());
      }
      BuiltinPluginName::WebWorkerTemplatePlugin => {
        web_worker_template_plugin(plugins);
      }
      BuiltinPluginName::MergeDuplicateChunksPlugin => {
        plugins.push(MergeDuplicateChunksPlugin.boxed());
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
      BuiltinPluginName::NamedModuleIdsPlugin => {
        plugins.push(NamedModuleIdsPlugin::default().boxed())
      }
      BuiltinPluginName::DeterministicModuleIdsPlugin => {
        plugins.push(DeterministicModuleIdsPlugin::default().boxed())
      }
      BuiltinPluginName::NamedChunkIdsPlugin => {
        plugins.push(NamedChunkIdsPlugin::new(None, None).boxed())
      }
      BuiltinPluginName::DeterministicChunkIdsPlugin => {
        plugins.push(DeterministicChunkIdsPlugin::default().boxed())
      }
      BuiltinPluginName::RealContentHashPlugin => plugins.push(RealContentHashPlugin.boxed()),
      BuiltinPluginName::RemoveEmptyChunksPlugin => plugins.push(RemoveEmptyChunksPlugin.boxed()),
      BuiltinPluginName::EnsureChunkConditionsPlugin => {
        plugins.push(EnsureChunkConditionsPlugin.boxed())
      }
      BuiltinPluginName::WarnCaseSensitiveModulesPlugin => {
        plugins.push(WarnCaseSensitiveModulesPlugin.boxed())
      }
      BuiltinPluginName::DataUriPlugin => plugins.push(DataUriPlugin.boxed()),
      BuiltinPluginName::FileUriPlugin => plugins.push(FileUriPlugin.boxed()),
      BuiltinPluginName::RuntimePlugin => plugins.push(RuntimePlugin.boxed()),
      BuiltinPluginName::JsonModulesPlugin => plugins.push(JsonPlugin.boxed()),
      BuiltinPluginName::InferAsyncModulesPlugin => plugins.push(InferAsyncModulesPlugin.boxed()),
      BuiltinPluginName::JavascriptModulesPlugin => plugins.push(JsPlugin::new().boxed()),
      BuiltinPluginName::AsyncWebAssemblyModulesPlugin => {
        plugins.push(AsyncWasmPlugin::new().boxed())
      }
      BuiltinPluginName::AssetModulesPlugin => plugins.push(AssetPlugin.boxed()),
      BuiltinPluginName::SourceMapDevToolPlugin => {
        let options: SourceMapDevToolPluginOptions =
          downcast_into::<RawSourceMapDevToolPluginOptions>(self.options)?.into();
        plugins.push(
          SourceMapDevToolModuleOptionsPlugin::new(SourceMapDevToolModuleOptionsPluginOptions {
            module: options.module,
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
        plugins.push(SwcCssMinimizerRspackPlugin {}.boxed())
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
      BuiltinPluginName::CssExtractPlugin => {
        let plugin = rspack_plugin_extract_css::plugin::PluginCssExtract::new(
          downcast_into::<RawCssExtractPluginOption>(self.options)?.into(),
        )
        .boxed();
        plugins.push(plugin);
      }
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  rspack_napi_shared::downcast_into(o).into_rspack_result()
}

// TO BE DEPRECATED
pub use raw_to_be_deprecated::RawBuiltins;
