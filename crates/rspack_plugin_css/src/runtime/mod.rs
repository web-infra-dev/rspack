use std::borrow::Cow;

use rspack_collections::Identifier;
use rspack_core::{
  basic_function, compile_boolean_matcher, impl_runtime_module,
  rspack_sources::{BoxSource, ConcatSource, RawSource, SourceExt},
  BooleanMatcher, ChunkUkey, Compilation, CrossOriginLoading, RuntimeGlobals, RuntimeModule,
  RuntimeModuleStage,
};
use rspack_plugin_runtime::{chunk_has_css, get_chunk_runtime_requirements, stringify_chunks};
use rustc_hash::FxHashSet as HashSet;

#[impl_runtime_module]
#[derive(Debug)]
pub struct CssLoadingRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for CssLoadingRuntimeModule {
  fn default() -> Self {
    Self::with_default(Identifier::from("webpack/runtime/css_loading"), None)
  }
}

impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> rspack_error::Result<BoxSource> {
    if let Some(chunk_ukey) = self.chunk {
      let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
      let runtime_requirements = get_chunk_runtime_requirements(compilation, &chunk_ukey);

      let unique_name = &compilation.options.output.unique_name;
      let with_hmr = runtime_requirements.contains(RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS);
      let with_fetch_priority = runtime_requirements.contains(RuntimeGlobals::HAS_FETCH_PRIORITY);

      let condition_map =
        compilation
          .chunk_graph
          .get_chunk_condition_map(&chunk_ukey, compilation, chunk_has_css);
      let has_css_matcher = compile_boolean_matcher(&condition_map);

      let with_loading = runtime_requirements.contains(RuntimeGlobals::ENSURE_CHUNK_HANDLERS)
        && !matches!(has_css_matcher, BooleanMatcher::Condition(false));

      let initial_chunks = chunk.get_all_initial_chunks(&compilation.chunk_group_by_ukey);
      let mut initial_chunk_ids_with_css = HashSet::default();
      let mut initial_chunk_ids_without_css = HashSet::default();
      for chunk_ukey in initial_chunks.iter() {
        let id = compilation
          .chunk_by_ukey
          .expect_get(chunk_ukey)
          .expect_id()
          .to_string();
        if chunk_has_css(chunk_ukey, compilation) {
          initial_chunk_ids_with_css.insert(id);
        } else {
          initial_chunk_ids_without_css.insert(id);
        }
      }

      if !with_hmr && !with_loading && initial_chunk_ids_with_css.is_empty() {
        return Ok(RawSource::from("").boxed());
      }

      let mut source = ConcatSource::default();
      // object to store loaded and loading chunks
      // undefined = chunk not loaded, null = chunk preloaded/prefetched
      // [resolve, reject, Promise] = chunk loading, 0 = chunk loaded

      // One entry initial chunk maybe is other entry dynamic chunk, so here
      // only render chunk without css. See packages/rspack/tests/runtimeCases/runtime/split-css-chunk test.
      source.add(RawSource::from(format!(
        "var installedChunks = {};\n",
        &stringify_chunks(&initial_chunk_ids_without_css, 0)
      )));

      let cross_origin_content = if let CrossOriginLoading::Enable(cross_origin) =
        &compilation.options.output.cross_origin_loading
      {
        if cross_origin == "use-credentials" {
          "link.crossOrigin = \"use-credentials\";".to_string()
        } else {
          format!(
            r#"
            if (link.href.indexOf(window.location.origin + '/') !== 0) {{
              link.crossOrigin = "{cross_origin}";
            }}
            "#
          )
        }
      } else {
        "".to_string()
      };

      let chunk_load_timeout = compilation.options.output.chunk_load_timeout.to_string();
      let environment = &compilation.options.output.environment;
      let with_compression = compilation.options.output.css_head_data_compression;

      let load_css_chunk_data = basic_function(
        environment,
        "target, link, chunkId",
        &format!(
          r#"var data, token = "", token2 = "", token3 = "", exports = {{}}, composes = [], {}name = "--webpack-" + uniqueName + "-" + chunkId, i, cc = 1, composes = {{}};
try {{
  if(!link) link = loadStylesheet(chunkId);
  var cssRules = link.sheet.cssRules || link.sheet.rules;
  var j = cssRules.length - 1;
  while(j > -1 && !data) {{
    var style = cssRules[j--].style;
    if(!style) continue;
    data = style.getPropertyValue(name);
  }}
}} catch(_) {{}}
if(!data) {{
  data = getComputedStyle(document.head).getPropertyValue(name);
}}
if(!data) return [];
{}
for(i = 0; cc; i++) {{
  cc = data.charCodeAt(i);
  if(cc == 58) {{ token2 = token; token = ""; }}
  else if(cc == 47) {{ token = token.replace(/^_/, ""); token2 = token2.replace(/^_/, ""); if (token3) {{ composes.push(token2, token3, token) }} else {{ exports[token2] = token }} token = ""; token2 = ""; token3 = "" }}
  else if(cc == 38) {{ {} }}
  else if(!cc || cc == 44) {{ token = token.replace(/^_/, ""); target[token] = ({}).bind(null, exports, composes); {}token = ""; token2 = ""; exports = {{}}; composes = [] }}
  else if(cc == 92) {{ token += data[++i] }}
  else if(cc == 64) {{ token3 = token; token = ""; }}
  else {{ token += data[i]; }}
}}
{}installedChunks[chunkId] = 0;
{}
"#,
          with_hmr.then(|| "moduleIds = [], ").unwrap_or_default(),
          if with_compression {
            r#"var map = {}, char = data[0], oldPhrase = char, decoded = char, code = 256, maxCode = "\uffff".charCodeAt(0), phrase;
              for (i = 1; i < data.length; i++) {
                cc = data[i].charCodeAt(0);
                if (cc < 256) phrase = data[i]; else phrase = map[cc] ? map[cc] : (oldPhrase + char);
                decoded += phrase;
                char = phrase.charAt(0);
                map[code] = oldPhrase + char;
                if (++code > maxCode) { code = 256; map = {}; }
                oldPhrase = phrase;
              }
              data = decoded;"#
          } else {
            "// css head data compression is disabled"
          },
          RuntimeGlobals::MAKE_NAMESPACE_OBJECT,
          basic_function(
            environment,
            "exports, composes, module",
            "handleCssComposes(exports, composes)\nmodule.exports = exports;"
          ),
          with_hmr
            .then(|| "moduleIds.push(token); ")
            .unwrap_or_default(),
          with_hmr
            .then(|| format!("if(target == {})", RuntimeGlobals::MODULE_FACTORIES))
            .unwrap_or_default(),
          with_hmr.then(|| "return moduleIds;").unwrap_or_default()
        ),
      );
      let load_initial_chunk_data = if initial_chunk_ids_with_css.len() > 2 {
        Cow::Owned(format!(
          "[{}].forEach(loadCssChunkData.bind(null, {}, 0));",
          initial_chunk_ids_with_css
            .iter()
            .map(|id| serde_json::to_string(id).expect("should ok to convert to string"))
            .collect::<Vec<_>>()
            .join(","),
          RuntimeGlobals::MODULE_FACTORIES
        ))
      } else if initial_chunk_ids_with_css.len() > 0 {
        Cow::Owned(
          initial_chunk_ids_with_css
            .iter()
            .map(|id| {
              let id = serde_json::to_string(id).expect("should ok to convert to string");
              format!(
                "loadCssChunkData({}, 0, {});",
                RuntimeGlobals::MODULE_FACTORIES,
                id
              )
            })
            .collect::<Vec<_>>()
            .join(""),
        )
      } else {
        Cow::Borrowed("// no initial css")
      };

      source.add(RawSource::from(
        include_str!("./css_loading.js")
          .replace(
            "__CROSS_ORIGIN_LOADING_PLACEHOLDER__",
            &cross_origin_content,
          )
          .replace("__CSS_CHUNK_DATA__", &load_css_chunk_data)
          .replace("__CHUNK_LOAD_TIMEOUT_PLACEHOLDER__", &chunk_load_timeout)
          .replace("__UNIQUE_NAME__", unique_name)
          .replace("__INITIAL_CSS_CHUNK_DATA__", &load_initial_chunk_data),
      ));

      if with_loading {
        let chunk_loading_global_expr = format!(
          "{}['{}']",
          &compilation.options.output.global_object,
          &compilation.options.output.chunk_loading_global
        );
        source.add(RawSource::from(
          include_str!("./css_loading_with_loading.js")
            .replace("$CHUNK_LOADING_GLOBAL_EXPR$", &chunk_loading_global_expr)
            .replace("CSS_MATCHER", &has_css_matcher.render("chunkId"))
            .replace(
              "$FETCH_PRIORITY$",
              if with_fetch_priority {
                ", fetchPriority"
              } else {
                ""
              },
            ),
        ));
      }

      if with_hmr {
        source.add(RawSource::from(include_str!("./css_loading_with_hmr.js")));
      }

      Ok(source.boxed())
    } else {
      unreachable!("should attach chunk for css_loading")
    }
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }
}
