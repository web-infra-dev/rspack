use hashlink::{LinkedHashMap, LinkedHashSet};
use itertools::Itertools;
use rspack_core::{
  impl_runtime_module,
  rspack_sources::{BoxSource, RawSource, SourceExt},
  ChunkUkey, Compilation, RuntimeGlobals, RuntimeModule, SourceType,
};
use rspack_identifier::Identifier;
use rustc_hash::FxHashMap;

use super::provide_shared_plugin::ProvideVersion;
use crate::utils::json_stringify;

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
      LinkedHashMap<DataInitStage, LinkedHashSet<DataInitInfo>>,
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
    let scope_to_data_init = init_per_scope
      .into_iter()
      .sorted_unstable_by_key(|(scope, _)| scope.to_string())
      .map(|(scope, stages)| {
        let stages: Vec<String> = stages
          .into_iter()
          .sorted_unstable_by_key(|(stage, _)| *stage)
          .flat_map(|(_, inits)| inits)
          .map(|info| match info {
            DataInitInfo::ExternalModuleId(Some(id)) => json_stringify(&id),
            DataInitInfo::ProvideSharedInfo(info) => {
              format!(
                "[{}, {}, {}, {}]",
                json_stringify(&info.name),
                json_stringify(&info.version.to_string()),
                info.factory,
                if info.eager { "1" } else { "0" }
              )
            }
            _ => "".to_string(),
          })
          .collect();
        format!("{}: [{}]", json_stringify(&scope), stages.join(", "))
      })
      .collect::<Vec<_>>()
      .join(", ");
    RawSource::from(format!(
      r#"
{share_scope_map} = {{}};
__webpack_require__.MF.initializeSharingData = {{ scopeToSharingDataMapping: {{ {scope_to_data_init} }}, uniqueName: {unique_name} }};
{initialize_sharing} = function(name, initScope) {{ return {initialize_sharing_fn}(name, initScope); }};
"#,
      share_scope_map = RuntimeGlobals::SHARE_SCOPE_MAP,
      scope_to_data_init = scope_to_data_init,
      unique_name = json_stringify(&compilation.options.output.unique_name),
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
  pub init: DataInitInfo,
}

pub type DataInitStage = i8;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DataInitInfo {
  ExternalModuleId(Option<String>),
  ProvideSharedInfo(ProvideSharedInfo),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProvideSharedInfo {
  pub name: String,
  pub version: ProvideVersion,
  pub factory: String,
  pub eager: bool,
}
