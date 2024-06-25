use std::hash::Hash;

use dashmap::DashMap;
use derivative::Derivative;
use once_cell::sync::Lazy;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesInlineInRuntimeBailout,
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
};
use rustc_hash::FxHashSet as HashSet;
use serde_json::json;

use crate::{
  module_filename_helpers::ModuleFilenameHelpers, ModuleFilenameTemplate, ModuleOrSource,
};

#[derive(Derivative, Clone)]
#[derivative(Debug)]
pub struct EvalDevToolModulePluginOptions {
  pub namespace: Option<String>,
  #[derivative(Debug = "ignore")]
  pub module_filename_template: Option<ModuleFilenameTemplate>,
  pub source_url_comment: Option<String>,
}

static EVAL_MODULE_RENDER_CACHE: Lazy<DashMap<BoxSource, BoxSource>> = Lazy::new(DashMap::default);

const EVAL_DEV_TOOL_MODULE_PLUGIN_NAME: &str = "rspack.EvalDevToolModulePlugin";

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EvalDevToolModulePlugin {
  namespace: String,
  source_url_comment: String,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
}

impl EvalDevToolModulePlugin {
  pub fn new(options: EvalDevToolModulePluginOptions) -> Self {
    let namespace = options.namespace.unwrap_or("".to_string());

    let source_url_comment = options
      .source_url_comment
      .unwrap_or("\n//# sourceURL=[url]".to_string());

    let module_filename_template =
      options
        .module_filename_template
        .unwrap_or(ModuleFilenameTemplate::String(
          "webpack://[namespace]/[resource-path]?[hash]".to_string(),
        ));

    Self::new_inner(namespace, source_url_comment, module_filename_template)
  }
}

#[plugin_hook(CompilerCompilation for EvalDevToolModulePlugin)]
async fn eval_devtool_plugin_compilation(
  &self,
  compilation: &mut Compilation,
  _params: &mut CompilationParams,
) -> Result<()> {
  let mut hooks = JsPlugin::get_compilation_hooks_mut(compilation);
  hooks
    .render_module_content
    .tap(eval_devtool_plugin_render_module_content::new(self));
  hooks
    .chunk_hash
    .tap(eval_devtool_plugin_js_chunk_hash::new(self));
  hooks
    .inline_in_runtime_bailout
    .tap(eval_devtool_plugin_inline_in_runtime_bailout::new(self));
  Ok(())
}

#[plugin_hook(JavascriptModulesRenderModuleContent for EvalDevToolModulePlugin)]
fn eval_devtool_plugin_render_module_content(
  &self,
  compilation: &Compilation,
  module: &BoxModule,
  render_source: &mut RenderSource,
  _init_fragments: &mut ChunkInitFragments,
) -> Result<()> {
  let origin_source = render_source.source.clone();
  if let Some(cached) = EVAL_MODULE_RENDER_CACHE.get(&origin_source) {
    render_source.source = cached.value().clone();
    return Ok(());
  } else if module.as_external_module().is_some() {
    return Ok(());
  }

  let output_options = &compilation.options.output;
  let str = match &self.module_filename_template {
    ModuleFilenameTemplate::String(s) => ModuleFilenameHelpers::create_filename_of_string_template(
      &ModuleOrSource::Module(module.identifier()),
      compilation,
      s,
      output_options,
      &self.namespace,
    ),
    ModuleFilenameTemplate::Fn(f) => {
      futures::executor::block_on(ModuleFilenameHelpers::create_filename_of_fn_template(
        &ModuleOrSource::Module(module.identifier()),
        compilation,
        f,
        output_options,
        &self.namespace,
      ))?
    }
  };
  let source = {
    let source = &origin_source.source();
    let footer = format!(
      "\n{}",
      self.source_url_comment.replace(
        "[url]",
        encode_uri(&str)
          .replace("%2F", "/")
          .replace("%20", "_")
          .replace("%5E", "^")
          .replace("%5C", "\\")
          .trim_start_matches('/')
      )
    );
    // TODO: Implement support for the trustedTypes option.
    // This will depend on the additionalModuleRuntimeRequirements hook.
    RawSource::from(format!("eval({});", json!(format!("{source}{footer}")))).boxed()
  };

  EVAL_MODULE_RENDER_CACHE.insert(origin_source, source.clone());
  render_source.source = source;
  Ok(())
}

#[plugin_hook(JavascriptModulesChunkHash for EvalDevToolModulePlugin)]
async fn eval_devtool_plugin_js_chunk_hash(
  &self,
  _compilation: &Compilation,
  _chunk_ukey: &ChunkUkey,
  hasher: &mut RspackHash,
) -> Result<()> {
  EVAL_DEV_TOOL_MODULE_PLUGIN_NAME.hash(hasher);
  Ok(())
}

#[plugin_hook(JavascriptModulesInlineInRuntimeBailout for EvalDevToolModulePlugin)]
fn eval_devtool_plugin_inline_in_runtime_bailout(
  &self,
  _compilation: &Compilation,
) -> Result<Option<String>> {
  Ok(Some("the eval devtool is used.".to_string()))
}

impl Plugin for EvalDevToolModulePlugin {
  fn name(&self) -> &'static str {
    EVAL_DEV_TOOL_MODULE_PLUGIN_NAME
  }

  fn apply(
    &self,
    ctx: PluginContext<&mut ApplyContext>,
    _options: &mut CompilerOptions,
  ) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(eval_devtool_plugin_compilation::new(self));
    Ok(())
  }
}

// https://tc39.es/ecma262/#sec-encodeuri-uri
fn encode_uri(uri: &str) -> String {
  encode(uri, ";/?:@&=+$,#")
}

static ALWAYS_UNESCAPED: Lazy<HashSet<char>> = Lazy::new(|| {
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~!*'()"
    .chars()
    .collect()
});

// https://tc39.es/ecma262/#sec-encode
fn encode(string: &str, extra_unescaped: &str) -> String {
  // Let R be the empty String.
  let mut r = String::new();
  // Let alwaysUnescaped be the string-concatenation of the ASCII word characters and "-.!~*'()".
  let always_unescaped = ALWAYS_UNESCAPED.clone();
  // Let unescapedSet be the string-concatenation of alwaysUnescaped and extraUnescaped.
  let unescaped_set: HashSet<char> = always_unescaped
    .union(&extra_unescaped.chars().collect::<HashSet<_>>())
    .cloned()
    .collect();
  for c in string.chars() {
    if unescaped_set.contains(&c) {
      r.push(c);
    } else {
      let mut b = [0u8; 4];
      let octets = c.encode_utf8(&mut b).as_bytes().to_vec();
      for octet in octets {
        r.push_str(&format!("%{:02X}", octet));
      }
    }
  }
  r
}

#[cfg(test)]
mod test {
  use super::*;

  // https://github.com/tc39/test262/blob/c47b716e8d6bea0c4510d449fd22b7ed5f8b0151/test/built-ins/encodeURI/S15.1.3.3_A4_T2.js#L6
  #[test]
  fn check_russian_alphabet() {
    assert_eq!(
      encode_uri("http://ru.wikipedia.org/wiki/Юникод"),
      "http://ru.wikipedia.org/wiki/%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4"
    );
    assert_eq!(
      encode_uri("http://ru.wikipedia.org/wiki/Юникод#Ссылки"),
      "http://ru.wikipedia.org/wiki/%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4#%D0%A1%D1%81%D1%8B%D0%BB%D0%BA%D0%B8"
    );
    assert_eq!(
      encode_uri("http://ru.wikipedia.org/wiki/Юникод#Версии Юникода"),
      "http://ru.wikipedia.org/wiki/%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4#%D0%92%D0%B5%D1%80%D1%81%D0%B8%D0%B8%20%D0%AE%D0%BD%D0%B8%D0%BA%D0%BE%D0%B4%D0%B0"
    );
  }
}
