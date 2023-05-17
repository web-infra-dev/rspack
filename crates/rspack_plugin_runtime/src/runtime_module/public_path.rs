use rspack_core::{
  get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, OutputOptions, PathData, PublicPath, RuntimeGlobals, RuntimeModule,
  SourceType,
};
use rspack_identifier::Identifier;

use super::utils::get_undo_path;
use crate::impl_runtime_module;

#[derive(Debug, Eq)]
pub struct PublicPathRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for PublicPathRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/public_path"),
      chunk: None,
    }
  }
}

impl RuntimeModule for PublicPathRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    match &compilation.options.output.public_path {
      PublicPath::String(str) => RawSource::from(
        include_str!("runtime/public_path.js").replace("__PUBLIC_PATH_PLACEHOLDER__", str),
      )
      .boxed(),
      PublicPath::Auto => {
        let chunk = compilation
          .chunk_by_ukey
          .get(&self.chunk.expect("The chunk should be attached."))
          .expect("Chunk is not found, make sure you had attach chunkUkey successfully.");
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
              .map(|i| i.as_str()),
          ),
        );
        RawSource::from(auto_public_path_template(
          &filename,
          &compilation.options.output,
        ))
        .boxed()
      }
    }
  }
}

// TODO: should use `__webpack_require__.g`
const GLOBAL: &str = "self";

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
  format!(
    r#"
  var scriptUrl;
  if ({GLOBAL}.importScripts) scriptUrl = {GLOBAL}.location + "";
  var document = {GLOBAL}.document;
  if (!scriptUrl && document) {{
    if (document.currentScript) scriptUrl = document.currentScript.src;
      if (!scriptUrl) {{
        var scripts = document.getElementsByTagName("script");
		    if (scripts.length) scriptUrl = scripts[scripts.length - 1].src;
      }}
    }}
  // When supporting browsers where an automatic publicPath is not supported you must specify an output.publicPath manually via configuration",
  // or pass an empty string ("") and set the __webpack_public_path__ variable from your code to use your own logic.',
  if (!scriptUrl) throw new Error("Automatic publicPath is not supported in this browser");
  scriptUrl = scriptUrl.replace(/#.*$/, "").replace(/\?.*$/, "").replace(/\/[^\/]+$/, "/");
  {assign}
  "#
  )
}

impl_runtime_module!(PublicPathRuntimeModule);
