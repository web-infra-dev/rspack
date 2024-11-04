use std::sync::LazyLock;
use std::{borrow::Cow, hash::Hash};

use cow_utils::CowUtils;
use dashmap::DashMap;
use derivative::Derivative;
use rspack_collections::UkeySet;
use rspack_core::{
  rspack_sources::{BoxSource, RawSource, Source, SourceExt},
  ApplyContext, BoxModule, ChunkInitFragments, ChunkUkey, Compilation, CompilationParams,
  CompilerCompilation, CompilerOptions, Plugin, PluginContext,
};
use rspack_core::{CompilationAdditionalTreeRuntimeRequirements, RuntimeGlobals};
use rspack_error::Result;
use rspack_hash::RspackHash;
use rspack_hook::{plugin, plugin_hook};
use rspack_plugin_javascript::{
  JavascriptModulesChunkHash, JavascriptModulesInlineInRuntimeBailout,
  JavascriptModulesRenderModuleContent, JsPlugin, RenderSource,
};

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

const EVAL_DEV_TOOL_MODULE_PLUGIN_NAME: &str = "rspack.EvalDevToolModulePlugin";

#[plugin]
#[derive(Derivative)]
#[derivative(Debug)]
pub struct EvalDevToolModulePlugin {
  namespace: String,
  source_url_comment: String,
  #[derivative(Debug = "ignore")]
  module_filename_template: ModuleFilenameTemplate,
  cache: DashMap<BoxSource, BoxSource>,
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

    Self::new_inner(
      namespace,
      source_url_comment,
      module_filename_template,
      Default::default(),
    )
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
  if let Some(cached_source) = self.cache.get(&origin_source) {
    render_source.source = cached_source.value().clone();
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
      &self.source_url_comment.cow_replace(
        "[url]",
        encode_uri(&str)
          .cow_replace("%2F", "/")
          .cow_replace("%20", "_")
          .cow_replace("%5E", "^")
          .cow_replace("%5C", "\\")
          .trim_start_matches('/')
      )
    );

    let module_content =
      simd_json::to_string(&format!("{source}{footer}")).expect("failed to parse string");
    RawSource::from(format!(
      "eval({});",
      if compilation.options.output.trusted_types.is_some() {
        format!("{}({})", RuntimeGlobals::CREATE_SCRIPT, module_content)
      } else {
        module_content
      }
    ))
    .boxed()
  };

  self.cache.insert(origin_source, source.clone());
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

  fn apply(&self, ctx: PluginContext<&mut ApplyContext>, _options: &CompilerOptions) -> Result<()> {
    ctx
      .context
      .compiler_hooks
      .compilation
      .tap(eval_devtool_plugin_compilation::new(self));
    ctx
      .context
      .compilation_hooks
      .additional_tree_runtime_requirements
      .tap(eval_devtool_plugin_additional_tree_runtime_requirements::new(self));
    Ok(())
  }
}

#[plugin_hook(CompilationAdditionalTreeRuntimeRequirements for EvalDevToolModulePlugin)]
async fn eval_devtool_plugin_additional_tree_runtime_requirements(
  &self,
  compilation: &mut Compilation,
  _chunk_ukey: &ChunkUkey,
  runtime_requirements: &mut RuntimeGlobals,
) -> Result<()> {
  if compilation.options.output.trusted_types.is_some() {
    runtime_requirements.insert(RuntimeGlobals::CREATE_SCRIPT);
  }

  Ok(())
}

// https://tc39.es/ecma262/#sec-encode
// UNESCAPED is combined by ALWAYS_UNESCAPED and ";/?:@&=+$,#"
static UNESCAPED: LazyLock<UkeySet<u32>> = LazyLock::new(|| {
  "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-._~!*'();/?:@&=+$,#"
    .chars()
    .map(|c| c as u32)
    .collect()
});

// https://tc39.es/ecma262/#sec-encode
fn encode_uri(string: &str) -> Cow<str> {
  // Let R be the empty String.
  let mut r = Cow::Borrowed(string);
  // Let alwaysUnescaped be the string-concatenation of the ASCII word characters and "-.!~*'()".
  for (byte_idx, c) in string.char_indices() {
    if UNESCAPED.contains(&(c as u32)) {
      match r {
        Cow::Borrowed(_) => {
          continue;
        }
        Cow::Owned(mut inner) => {
          inner.push(c);
          r = Cow::Owned(inner);
        }
      }
    } else {
      match r {
        Cow::Borrowed(_) => {
          let mut s = string[0..byte_idx].to_string();
          let mut b = [0u8; 4];
          let octets = c.encode_utf8(&mut b).as_bytes().to_vec();
          for octet in octets {
            s.push_str(&format!("%{octet:02X}"));
          }
          r = Cow::Owned(s);
        }
        Cow::Owned(mut inner) => {
          let mut b = [0u8; 4];
          let octets = c.encode_utf8(&mut b).as_bytes().to_vec();
          for octet in octets {
            inner.push_str(&format!("%{octet:02X}"));
          }
          r = Cow::Owned(inner);
        }
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
