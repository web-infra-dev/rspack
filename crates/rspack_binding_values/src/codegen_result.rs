use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{get_runtime_key, CodeGenerationResult, CodeGenerationResults};

#[napi(object)]
#[derive(Debug)]
pub struct JsCodegenerationResults {
  pub map: HashMap<String, HashMap<String, JsCodegenerationResult>>,
}

#[napi(object)]
#[derive(Debug)]
pub struct JsCodegenerationResult {
  pub sources: HashMap<String, String>,
}

impl From<CodeGenerationResult> for JsCodegenerationResult {
  fn from(result: CodeGenerationResult) -> Self {
    Self {
      sources: result
        .inner
        .into_iter()
        .map(|(source_type, source)| (source_type.to_string(), source.source().to_string()))
        .collect(),
    }
  }
}

impl From<CodeGenerationResults> for JsCodegenerationResults {
  fn from(results: CodeGenerationResults) -> Self {
    let id_result_map = results.module_generation_result_map;

    Self {
      map: results
        .map
        .into_iter()
        .map(|(module_id, runtime_result_map)| {
          let mut runtime_map: HashMap<String, JsCodegenerationResult> = Default::default();
          match &runtime_result_map.mode {
            rspack_core::RuntimeMode::Empty => {}
            rspack_core::RuntimeMode::SingleEntry => {
              runtime_map.insert(
                get_runtime_key(runtime_result_map.single_runtime.expect("exist")),
                id_result_map
                  .get(&runtime_result_map.single_value.expect("TODO"))
                  .expect("TODO")
                  .clone()
                  .into(),
              );
            }
            rspack_core::RuntimeMode::Map => {
              runtime_result_map.map.into_iter().for_each(|(k, v)| {
                runtime_map.insert(k, id_result_map.get(&v).expect("TODO").clone().into());
              });
            }
          };

          (module_id.to_string(), runtime_map)
        })
        .collect(),
    }
  }
}
