mod raw_banner;
mod raw_copy;
mod raw_html;
mod raw_limit_chunk_count;
mod raw_mf;
mod raw_progress;
mod raw_swc_js_minimizer;
mod raw_to_be_deprecated;

use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue},
  JsUnknown,
};
use napi_derive::napi;
use rspack_core::{
  mf::{
    consume_shared_plugin::ConsumeSharedPlugin, container_plugin::ContainerPlugin,
    container_reference_plugin::ContainerReferencePlugin,
    module_federation_runtime_plugin::ModuleFederationRuntimePlugin,
    provide_shared_plugin::ProvideSharedPlugin,
  },
  BoxPlugin, Define, DefinePlugin, PluginExt, Provide, ProvidePlugin,
};
use rspack_error::Result;
use rspack_napi_shared::NapiResultExt;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::{CopyRspackPlugin, CopyRspackPluginOptions};
use rspack_plugin_entry::EntryPlugin;
use rspack_plugin_externals::{
  electron_target_plugin, http_externals_rspack_plugin, node_target_plugin, ExternalsPlugin,
};
use rspack_plugin_hmr::HotModuleReplacementPlugin;
use rspack_plugin_html::HtmlRspackPlugin;
use rspack_plugin_library::enable_library_plugin;
use rspack_plugin_limit_chunk_count::LimitChunkCountPlugin;
use rspack_plugin_merge_duplicate_chunks::MergeDuplicateChunksPlugin;
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_runtime::{
  enable_chunk_loading_plugin, ArrayPushCallbackChunkFormatPlugin, CommonJsChunkFormatPlugin,
  ModuleChunkFormatPlugin,
};
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerRspackPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerRspackPlugin;
use rspack_plugin_wasm::enable_wasm_loading_plugin;
use rspack_plugin_web_worker_template::web_worker_template_plugin;

use self::raw_mf::{RawConsumeOptions, RawContainerReferencePluginOptions, RawProvideOptions};
pub use self::{
  raw_banner::RawBannerPluginOptions, raw_copy::RawCopyRspackPluginOptions,
  raw_html::RawHtmlRspackPluginOptions, raw_limit_chunk_count::RawLimitChunkCountPluginOptions,
  raw_mf::RawContainerPluginOptions, raw_progress::RawProgressPluginOptions,
  raw_swc_js_minimizer::RawSwcJsMinimizerRspackPluginOptions,
};
use crate::{
  RawEntryPluginOptions, RawExternalItemWrapper, RawExternalsPluginOptions,
  RawHttpExternalsRspackPluginOptions, RawOptionsApply, RawSplitChunksOptions,
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
  CommonJsChunkFormatPlugin,
  ArrayPushCallbackChunkFormatPlugin,
  ModuleChunkFormatPlugin,
  HotModuleReplacementPlugin,
  LimitChunkCountPlugin,
  WebWorkerTemplatePlugin,
  MergeDuplicateChunksPlugin,
  SplitChunksPlugin,
  OldSplitChunksPlugin,
  ContainerPlugin,
  ContainerReferencePlugin,
  ModuleFederationRuntimePlugin,
  ProvideSharedPlugin,
  ConsumeSharedPlugin,

  // rspack specific plugins
  HttpExternalsRspackPlugin,
  CopyRspackPlugin,
  HtmlRspackPlugin,
  SwcJsMinimizerRspackPlugin,
  SwcCssMinimizerRspackPlugin,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub name: BuiltinPluginName,
  pub options: JsUnknown,
}

impl RawOptionsApply for BuiltinPlugin {
  type Options = ();

  fn apply(
    self,
    plugins: &mut Vec<BoxPlugin>,
  ) -> std::result::Result<Self::Options, rspack_error::Error> {
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
      BuiltinPluginName::WebWorkerTemplatePlugin => {
        web_worker_template_plugin(plugins);
      }
      BuiltinPluginName::MergeDuplicateChunksPlugin => {
        plugins.push(MergeDuplicateChunksPlugin.boxed());
      }
      BuiltinPluginName::SplitChunksPlugin => {
        use rspack_plugin_split_chunks_new::SplitChunksPlugin;
        let options = downcast_into::<RawSplitChunksOptions>(self.options)?.into();
        plugins.push(SplitChunksPlugin::new(options).boxed());
      }
      BuiltinPluginName::OldSplitChunksPlugin => {
        use rspack_plugin_split_chunks::SplitChunksPlugin;
        let options = downcast_into::<RawSplitChunksOptions>(self.options)?.into();
        plugins.push(SplitChunksPlugin::new(options).boxed());
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
      BuiltinPluginName::ModuleFederationRuntimePlugin => {
        plugins.push(ModuleFederationRuntimePlugin::default().boxed())
      }
      BuiltinPluginName::ProvideSharedPlugin => {
        let mut provides: Vec<_> = downcast_into::<Vec<RawProvideOptions>>(self.options)?
          .into_iter()
          .map(Into::into)
          .collect();
        provides.sort_unstable_by_key(|(k, _)| k.to_string());
        plugins.push(ProvideSharedPlugin::new(provides).boxed())
      }
      BuiltinPluginName::ConsumeSharedPlugin => {
        let consumes: Vec<_> = downcast_into::<Vec<RawConsumeOptions>>(self.options)?
          .into_iter()
          .map(Into::into)
          .collect();
        plugins.push(ConsumeSharedPlugin::new(consumes).boxed())
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
    }
    Ok(())
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  rspack_napi_shared::downcast_into(o).into_rspack_result()
}

// TO BE DEPRECATED
pub use raw_to_be_deprecated::RawBuiltins;
