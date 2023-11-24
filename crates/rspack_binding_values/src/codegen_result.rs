use std::collections::HashMap;

use napi_derive::napi;
use rspack_core::{CodeGenerationResult, CodeGenerationResults};

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
          (
            module_id.to_string(),
            runtime_result_map
              .map
              .into_iter()
              .map(|(k, result_id)| {
                (
                  k,
                  id_result_map
                    .get(&result_id)
                    .expect("should exist codegenResult")
                    .clone()
                    .into(),
                )
              })
              .collect(),
          )
        })
        .collect(),
    }
  }
}
