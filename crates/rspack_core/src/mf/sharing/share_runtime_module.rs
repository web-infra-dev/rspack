use hashlink::{LinkedHashMap, LinkedHashSet};
use itertools::Itertools;
use rspack_identifier::Identifier;
use rspack_sources::{BoxSource, RawSource, SourceExt};
use rustc_hash::FxHashMap;

use crate::{
  impl_runtime_module, ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, SourceType,
};

#[derive(Debug, Eq)]
pub struct ShareRuntimeModule {
  id: Identifier,
  chunk: Option<ChunkUkey>,
}

impl Default for ShareRuntimeModule {
  fn default() -> Self {
    Self {
      id: Identifier::from("webpack/runtime/sharing"),
      chunk: None,
    }
  }
}

impl RuntimeModule for ShareRuntimeModule {
  fn name(&self) -> Identifier {
    self.id
  }

  fn generate(&self, compilation: &Compilation) -> BoxSource {
    let chunk_ukey = self
      .chunk
      .expect("should have chunk in <ShareRuntimeModule as RuntimeModule>::generate");
    let chunk = compilation.chunk_by_ukey.expect_get(&chunk_ukey);
    let mut init_per_scope: FxHashMap<
      String,
      LinkedHashMap<DataInitStage, LinkedHashSet<DataInit>>,
    > = FxHashMap::default();
    for c in chunk.get_all_referenced_chunks(&compilation.chunk_group_by_ukey) {
      let chunk = compilation.chunk_by_ukey.expect_get(&c);
      let modules = compilation
        .chunk_graph
        .get_chunk_modules_iterable_by_source_type(
          &c,
          SourceType::ShareInit,
          &compilation.module_graph,
        )
        .sorted_unstable_by_key(|m| m.identifier());
      for m in modules {
        let code_gen = compilation
          .code_generation_results
          .get(&m.identifier(), Some(&chunk.runtime)).expect("should have code_generation_result of share-init sourceType module at <ShareRuntimeModule as RuntimeModule>::generate");
        let Some(data) = code_gen.data.get::<CodeGenerationDataShareInit>() else {
          continue;
        };
        for item in &data.items {
          let (_, stages) = init_per_scope
            .raw_entry_mut()
            .from_key(&item.share_scope)
            .or_insert_with(|| (item.share_scope.to_owned(), LinkedHashMap::default()));
          let list = stages
            .entry(item.init_stage)
            .or_insert_with(LinkedHashSet::default);
          list.insert(item.init.clone());
        }
      }
    }
    let init_per_scope_body = init_per_scope
      .into_iter()
      .sorted_unstable_by_key(|(scope, _)| scope.to_string())
      .map(|(scope, stages)| {
        let stages = stages
          .into_iter()
          .sorted_unstable_by_key(|(stage, _)| *stage)
          .flat_map(|(_, inits)| {
            inits.into_iter().filter_map(|init| match init {
              DataInit::ExternalModuleId(Some(id)) => Some(format!(
                "initExternal({});",
                serde_json::to_string(&id).expect("module_id should able to json to_string")
              )),
              _ => None,
            })
          })
          .collect::<Vec<_>>()
          .join("\n");
        format!(
          r#"case {}: {{
{}
}}
break;"#,
          serde_json::to_string(&scope).expect("should able to json to_string"),
          stages
        )
      })
      .collect::<Vec<_>>()
      .join("\n");
    RawSource::from(format!(
      r#"
{share_scope_map} = {{}};
var initPromises = {{}};
var initTokens = {{}};
var initPerScope = function(name, register, initExternal) {{
  switch(name) {{
{init_per_scope_body}
  }}
}};
{initialize_sharing} = function(name, initScope) {{ return {initialize_sharing_fn}({{ name: name, initScope: initScope, initPerScope: initPerScope, initTokens: initTokens, initPromises: initPromises }}); }};
"#,
      share_scope_map = RuntimeGlobals::SHARE_SCOPE_MAP,
      init_per_scope_body = init_per_scope_body,
      initialize_sharing = RuntimeGlobals::INITIALIZE_SHARING,
      initialize_sharing_fn = "__webpack_require__.MF.initializeSharing"
    ))
    .boxed()
  }

  fn attach(&mut self, chunk: ChunkUkey) {
    self.chunk = Some(chunk);
  }
}

impl_runtime_module!(ShareRuntimeModule);

#[derive(Debug, Clone)]
pub struct CodeGenerationDataShareInit {
  pub items: Vec<ShareInitData>,
}

#[derive(Debug, Clone)]
pub struct ShareInitData {
  pub share_scope: String,
  pub init_stage: DataInitStage,
  pub init: DataInit,
}

pub type DataInitStage = i8;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum DataInit {
  ExternalModuleId(Option<String>),
}
