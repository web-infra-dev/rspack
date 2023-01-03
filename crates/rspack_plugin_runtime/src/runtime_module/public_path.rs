use rspack_core::{
  get_js_chunk_filename_template,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  runtime_globals::PUBLIC_PATH,
  ChunkUkey, Compilation, FilenameRenderOptions, OutputOptions, PublicPath, RuntimeModule,
};

#[derive(Debug, Default)]
pub struct PublicPathRuntimeModule {
  // FIXME: maybe it unnecessary
  pub chunk: ChunkUkey,
}

impl RuntimeModule for PublicPathRuntimeModule {
  fn identifier(&self) -> String {
    "webpack/runtime/public_path".to_string()
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
          .get(&self.chunk)
          .expect("Chunk not found");
        let filename = get_js_chunk_filename_template(
          chunk,
          &compilation.options.output,
          &compilation.chunk_group_by_ukey,
        );
        let hash = Some(chunk.get_render_hash());
        let filename = filename.render(FilenameRenderOptions {
          name: chunk.name_for_filename_template(),
          extension: Some(".js".to_string()),
          id: chunk.id.clone(),
          contenthash: hash.clone(),
          chunkhash: hash.clone(),
          hash,
          ..Default::default()
        });
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
    format!("{PUBLIC_PATH} = scriptUrl")
  } else {
    format!("{PUBLIC_PATH} = scriptUrl + '{undo_path}'")
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

fn get_undo_path(filename: &str, p: String, enforce_relative: bool) -> String {
  let mut depth: i32 = -1;
  let mut append = String::new();
  let mut p = p;
  if p.ends_with('/') || p.ends_with('\\') {
    p.pop();
  }
  for part in filename.split(&['/', '\\']) {
    if part == ".." {
      if depth > -1 {
        depth -= 1
      } else {
        let pos = match (p.rfind('/'), p.rfind('\\')) {
          (None, None) => {
            p.push('/');
            return p;
          }
          (None, Some(j)) => j,
          (Some(i), None) => i,
          (Some(i), Some(j)) => usize::max(i, j),
        };
        append = format!("{}/{append}", &p[pos + 1..]);
        p = p[0..pos].to_string();
      }
    } else if part != "." {
      depth += 1;
    }
  }

  if depth > 0 {
    format!("{}{append}", "../".repeat(depth as usize))
  } else if enforce_relative {
    format!("./{append}")
  } else {
    append
  }
}

#[test]
fn test_get_undo_path() {
  assert_eq!(get_undo_path("a", "/a/b/c".to_string(), true), "./");
  assert_eq!(
    get_undo_path("static/js/a.js", "/a/b/c".to_string(), false),
    "../../"
  );
}
