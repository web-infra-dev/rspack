use std::sync::Arc;

use rspack_core::{
  impl_runtime_module, rspack_sources::RawSource, ChunkUkey, Compilation, CrossOriginLoading,
  RuntimeGlobals, RuntimeModule, RuntimeModuleStage,
};
use rspack_error::Result;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::plugin::{InsertType, SOURCE_TYPE};

static RUNTIME_CODE: &str = include_str!("./runtime/css_load.js");
static WITH_LOADING: &str = include_str!("./runtime/with_loading.js");
static WITH_HMR: &str = include_str!("./runtime/with_hmr.js");

#[impl_runtime_module]
#[derive(Debug)]
pub(crate) struct CssLoadingRuntimeModule {
  chunk: ChunkUkey,
  attributes: FxHashMap<String, String>,
  link_type: Option<String>,
  insert: InsertType,

  loading: bool,
  hmr: bool,
}

impl CssLoadingRuntimeModule {
  pub(crate) fn new(
    chunk: ChunkUkey,
    attributes: FxHashMap<String, String>,
    link_type: Option<String>,
    insert: InsertType,
    loading: bool,
    hmr: bool,
  ) -> Self {
    Self::with_default(chunk, attributes, link_type, insert, loading, hmr)
  }

  fn get_css_chunks(&self, compilation: &Compilation) -> FxHashSet<ChunkUkey> {
    let mut set: FxHashSet<ChunkUkey> = Default::default();
    let module_graph = compilation.get_module_graph();

    let chunk = compilation.chunk_by_ukey.expect_get(&self.chunk);

    for chunk in chunk.get_all_async_chunks(&compilation.chunk_group_by_ukey) {
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(&chunk, SOURCE_TYPE[0], &module_graph);

      if modules.count() > 0 {
        set.insert(chunk);
      }
    }

    set
  }
}

impl RuntimeModule for CssLoadingRuntimeModule {
  fn name(&self) -> rspack_identifier::Identifier {
    "webpack/runtime/css loading".into()
  }

  fn stage(&self) -> RuntimeModuleStage {
    RuntimeModuleStage::Attach
  }

  fn generate(
    &self,
    compilation: &rspack_core::Compilation,
  ) -> Result<rspack_core::rspack_sources::BoxSource> {
    let runtime = RUNTIME_CODE;

    let mut attr = String::default();
    let mut attributes = self.attributes.iter().collect::<Vec<_>>();
    attributes.sort_unstable_by(|(k1, _), (k2, _)| k1.cmp(k2));

    for (attr_key, attr_value) in attributes {
      attr += &format!("linkTag.setAttribute({}, {});\n", attr_key, attr_value);
    }
    let runtime = runtime.replace("__SET_ATTRIBUTES__", &attr);

    let runtime = if let Some(link_type) = &self.link_type {
      runtime.replace("__SET_LINKTYPE__", &format!("linkTag.type={};", link_type))
    } else {
      runtime.replace("__SET_LINKTYPE__", "")
    };

    let runtime = if let CrossOriginLoading::Enable(cross_origin_loading) =
      &compilation.options.output.cross_origin_loading
    {
      runtime.replace(
        "__CROSS_ORIGIN_LOADING__",
        &format!(
          "if (linkTag.href.indexOf(window.location.origin + '/') !== 0) {{
  linkTag.crossOrigin = \"{}\";
}}",
          cross_origin_loading
        ),
      )
    } else {
      runtime.replace("__CROSS_ORIGIN_LOADING__", "")
    };

    let runtime = match &self.insert {
      InsertType::Fn(f) => runtime.replace("__INSERT__", &format!("({f})(linkTag);")),
      InsertType::Selector(sel) => runtime.replace(
        "__INSERT__",
        &format!("var target = document.querySelector({sel});\ntarget.parentNode.insertBefore(linkTag, target.nextSibling);"),
      ),
      InsertType::Default => runtime.replace(
        "__INSERT__",
        "if (oldTag) {
  oldTag.parentNode.insertBefore(linkTag, oldTag.nextSibling);
} else {
  document.head.appendChild(linkTag);
}",
      ),
    };

    let runtime = if self.loading {
      let chunks = self.get_css_chunks(compilation);
      if chunks.is_empty() {
        runtime.replace("__WITH_LOADING__", "// no chunk loading")
      } else {
        let chunk = compilation.chunk_by_ukey.expect_get(&self.chunk);
        let with_loading = WITH_LOADING.replace(
          "__INSTALLED_CHUNKS__",
          &chunk
            .ids
            .iter()
            .fold(String::default(), |output, id: &String| {
              format!(
                "{output}{}: 0,\n",
                serde_json::to_string(id).expect("json stringify failed")
              )
            }),
        );

        let with_loading = with_loading.replace(
          "__ENSURE_CHUNK_HANDLERS__",
          &RuntimeGlobals::ENSURE_CHUNK_HANDLERS.to_string(),
        );

        let with_loading = with_loading.replace(
          "__CSS_CHUNKS__",
          &format!(
            "{{\n{}\n}}",
            chunks
              .iter()
              .filter_map(|id| {
                let chunk = compilation.chunk_by_ukey.expect_get(id);

                chunk.id.as_ref().map(|id| {
                  format!(
                    "{}: 1,\n",
                    serde_json::to_string(id).expect("json stringify failed")
                  )
                })
              })
              .collect::<String>()
          ),
        );

        runtime.replace("__WITH_LOADING__", &with_loading)
      }
    } else {
      runtime.replace("__WITH_LOADING__", "// no chunk loading")
    };

    let runtime = if self.hmr {
      runtime.replace(
        "__WITH_HMT__",
        &WITH_HMR.replace(
          "__HMR_DOWNLOAD__",
          &RuntimeGlobals::HMR_DOWNLOAD_UPDATE_HANDLERS.to_string(),
        ),
      )
    } else {
      runtime.replace("__WITH_HMT__", "// no hmr")
    };

    Ok(Arc::new(RawSource::from(runtime)))
  }
}
