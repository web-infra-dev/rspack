mod raw_banner;
mod raw_copy;
mod raw_decorator;
mod raw_html;
mod raw_plugin_import;
mod raw_preset_env;
mod raw_progress;
mod raw_react;
mod raw_relay;
mod raw_swc_js_minimizer;

use napi::{
  bindgen_prelude::{FromNapiValue, ToNapiValue},
  JsUnknown,
};
use napi_derive::napi;
use rspack_core::{
  BoxPlugin, CopyPluginConfig, DecoratorOptionsPlugin, Define, DefinePlugin, EmotionPlugin,
  NoEmitAssetsPlugin, PluginExt, PluginImportPlugin, PresetEnvPlugin, Provide, ProvidePlugin,
  ReactOptionsPlugin, RelayPlugin, TreeShakingPlugin,
};
use rspack_error::{internal_error, Error, Result};
use rspack_napi_shared::NapiResultExt;
use rspack_plugin_banner::BannerPlugin;
use rspack_plugin_copy::CopyPlugin;
use rspack_plugin_dev_friendly_split_chunks::DevFriendlySplitChunksPlugin;
use rspack_plugin_html::HtmlPlugin;
use rspack_plugin_progress::ProgressPlugin;
use rspack_plugin_swc_css_minimizer::SwcCssMinimizerPlugin;
use rspack_plugin_swc_js_minimizer::SwcJsMinimizerPlugin;

pub use self::{
  raw_banner::RawBannerConfig, raw_copy::RawCopyConfig, raw_decorator::RawDecoratorOptions,
  raw_html::RawHtmlPluginConfig, raw_plugin_import::RawPluginImportConfig,
  raw_preset_env::RawPresetEnv, raw_progress::RawProgressPluginConfig, raw_react::RawReactOptions,
  raw_relay::RawRelayConfig, raw_swc_js_minimizer::RawMinification,
};

#[napi]
pub enum BuiltinPluginKind {
  Define,
  Provide,
  Banner,
  SwcJsMinimizer,
  SwcCssMinimizer,
  PresetEnv,
  TreeShaking,
  ReactOptions,
  DecoratorOptions,
  NoEmitAssets,
  Emotion,
  Relay,
  PluginImport,
  DevFriendlySplitChunks,
  Progress,
  Copy,
  Html,
}

#[napi(object)]
pub struct BuiltinPlugin {
  pub kind: BuiltinPluginKind,
  pub options: JsUnknown,
}

impl TryFrom<BuiltinPlugin> for BoxPlugin {
  type Error = Error;

  fn try_from(value: BuiltinPlugin) -> Result<Self> {
    let plugin = match value.kind {
      BuiltinPluginKind::Define => {
        DefinePlugin::new(downcast_into::<Define>(value.options)?).boxed()
      }
      BuiltinPluginKind::Provide => {
        ProvidePlugin::new(downcast_into::<Provide>(value.options)?).boxed()
      }
      BuiltinPluginKind::Banner => {
        BannerPlugin::new(downcast_into::<RawBannerConfig>(value.options)?.try_into()?).boxed()
      }
      BuiltinPluginKind::SwcJsMinimizer => {
        SwcJsMinimizerPlugin::new(downcast_into::<RawMinification>(value.options)?.try_into()?)
          .boxed()
      }
      BuiltinPluginKind::SwcCssMinimizer => SwcCssMinimizerPlugin {}.boxed(),
      BuiltinPluginKind::PresetEnv => {
        PresetEnvPlugin::new(downcast_into::<RawPresetEnv>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::TreeShaking => {
        TreeShakingPlugin::new(downcast_into::<String>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::ReactOptions => {
        ReactOptionsPlugin::new(downcast_into::<RawReactOptions>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::DecoratorOptions => {
        DecoratorOptionsPlugin::new(downcast_into::<RawDecoratorOptions>(value.options)?.into())
          .boxed()
      }
      BuiltinPluginKind::NoEmitAssets => {
        NoEmitAssetsPlugin::new(downcast_into::<bool>(value.options)?).boxed()
      }
      BuiltinPluginKind::Emotion => EmotionPlugin::new(
        serde_json::from_str(&downcast_into::<String>(value.options)?)
          .map_err(|e| internal_error!(e.to_string()))?,
      )
      .boxed(),
      BuiltinPluginKind::Relay => {
        RelayPlugin::new(downcast_into::<RawRelayConfig>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::PluginImport => PluginImportPlugin::new(
        downcast_into::<Vec<RawPluginImportConfig>>(value.options)?
          .into_iter()
          .map(|i| i.into())
          .collect(),
      )
      .boxed(),
      BuiltinPluginKind::DevFriendlySplitChunks => DevFriendlySplitChunksPlugin::new().boxed(),
      BuiltinPluginKind::Progress => {
        ProgressPlugin::new(downcast_into::<RawProgressPluginConfig>(value.options)?.into()).boxed()
      }
      BuiltinPluginKind::Copy => CopyPlugin::new(
        CopyPluginConfig::from(downcast_into::<RawCopyConfig>(value.options)?).patterns,
      )
      .boxed(),
      BuiltinPluginKind::Html => {
        HtmlPlugin::new(downcast_into::<RawHtmlPluginConfig>(value.options)?.into()).boxed()
      }
    };
    Ok(plugin)
  }
}

fn downcast_into<T: FromNapiValue + 'static>(o: JsUnknown) -> Result<T> {
  <T as FromNapiValue>::from_unknown(o).into_rspack_result()
}
