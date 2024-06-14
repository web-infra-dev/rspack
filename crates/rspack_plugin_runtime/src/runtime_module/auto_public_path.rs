use rspack_core::{
  get_js_chunk_filename_template, impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};
use rspack_identifier::Identifier;
use rspack_util::source_map::SourceMapKind;

use super::utils::get_undo_path;

#[impl_runtime_module]
#[derive(Debug, Eq)]
pub struct AutoPublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for AutoPublicPathRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/auto_public_path"),
      chunk: None,
      source_map_kind: SourceMapKind::empty(),
      custom_source: None,
    }
  }
}

impl RuntimeModule for AutoPublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    let chunk = self.chunk.expect("The chunk should be attached");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk);
    let filename = get_js_chunk_filename_template(
      chunk,
      &compilation.options.output,
      &compilation.chunk_group_by_ukey,
    );
    let filename = compilation.get_path(
      filename,
      PathData::default().chunk(chunk).content_hash_optional(
        chunk
          .content_hash
          .get(&SourceType::JavaScript)
          .map(|i| i.rendered(compilation.options.output.hash_digest_length)),
      ),
    )?;
    Ok(
      RawSource::from(auto_public_path_template(
        &filename,
        &compilation.options.output,
      ))
      .boxed(),
    )
  }
}

fn auto_public_path_template(filename: &str, output: &OutputOptions) -> String {
  let output_path = output.path.display().to_string();
  let undo_path = get_undo_path(filename, output_path, false);
  let assign = if undo_path.is_empty() {
    format!("{} = scriptUrl", RuntimeGlobals::PUBLIC_PATH)
  } else {
    format!(
      "{} = scriptUrl + '{undo_path}'",
      RuntimeGlobals::PUBLIC_PATH
    )
  };
  let global = RuntimeGlobals::GLOBAL.name();

  // TODO: replace import.meta with importMetaName
  let script_url_template = if output.script_type.eq("module") {
    r#"var scriptUrl;
    if (typeof import.meta.url === "string") scriptUrl = import.meta.url
    "#
    .to_string()
  } else {
    format!(
      r#"var scriptUrl;
    if ({global}.importScripts) scriptUrl = {global}.location + "";
    var document = {global}.document;
    if (!scriptUrl && document) {{
      if (document.currentScript) scriptUrl = document.currentScript.src;
        if (!scriptUrl) {{
          var scripts = document.getElementsByTagName("script");
              if (scripts.length) {{
                var i = scripts.length - 1;
                while (i > -1 && (!scriptUrl || !/^http(s?):/.test(scriptUrl))) scriptUrl = scripts[i--].src;
              }}
        }}
      }}
    "#
    )
  };
  format!(
    r#"
    {script_url_template}
    // When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration",
    // or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.',
    if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
    scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
    {assign}
    "#
  )
}
