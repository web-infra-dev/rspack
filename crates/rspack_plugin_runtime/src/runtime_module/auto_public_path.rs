use rspack_collections::Identifier;
use rspack_core::{
  get_js_chunk_filename_template, impl_runtime_module,
  rspack_sources::{BoxSource, RawStringSource, SourceExt},
  ChunkUkey, Compilation, OutputOptions, PathData, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage, SourceType,
};

use super::utils::get_undo_path;

#[impl_runtime_module]
#[derive(Debug)]
pub struct AutoPublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for AutoPublicPathRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/auto_public_path"), None)
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
      &filename,
      PathData::default()
        .chunk_id_optional(
          chunk
            .id(&compilation.chunk_ids_artifact)
            .map(|id| id.as_str()),
        )
        .chunk_hash_optional(chunk.rendered_hash(
          &compilation.chunk_hashes_artifact,
          compilation.options.output.hash_digest_length,
        ))
        .chunk_name_optional(chunk.name_for_filename_template(&compilation.chunk_ids_artifact))
        .content_hash_optional(chunk.rendered_content_hash_by_source_type(
          &compilation.chunk_hashes_artifact,
          &SourceType::JavaScript,
          compilation.options.output.hash_digest_length,
        )),
    )?;
    Ok(
      RawStringSource::from(auto_public_path_template(
        &filename,
        &compilation.options.output,
      ))
      .boxed(),
    )
  }
}

fn auto_public_path_template(filename: &str, output: &OutputOptions) -> String {
  let output_path = output.path.as_str().to_string();
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
  let import_meta_name = output.import_meta_name.clone();

  let script_url_template = if output.script_type.eq("module") {
    format!(
      r#"var scriptUrl;
    if (typeof {import_meta_name}.url === "string") scriptUrl = {import_meta_name}.url
    "#
    )
    .to_string()
  } else {
    format!(
      r#"var scriptUrl;
    if ({global}.importScripts) scriptUrl = {global}.location + "";
    var document = {global}.document;
    if (!scriptUrl && document) {{
      // Technically we could use `document.currentScript instanceof window.HTMLScriptElement`,
      // but an attacker could try to inject `<script>HTMLScriptElement = HTMLImageElement</script>`
      // and use `<img name="currentScript" src="https://attacker.controlled.server/"></img>`
      if (document.currentScript && document.currentScript.tagName.toUpperCase() === 'SCRIPT') scriptUrl = document.currentScript.src;
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
