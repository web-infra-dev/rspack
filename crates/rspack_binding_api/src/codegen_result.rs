use napi_derive::napi;
use rspack_core::{CodeGenerationResult, CodeGenerationResults, get_runtime_key};
use rustc_hash::FxHashMap as HashMap;

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

impl From<&CodeGenerationResult> for JsCodegenerationResult {
  fn from(result: &CodeGenerationResult) -> Self {
    Self {
      sources: result
        .inner
        .as_ref()
        .iter()
        .map(|(source_type, source)| {
          (
            source_type.to_string(),
            source.source().into_string_lossy().into_owned(),
          )
        })
        .collect::<HashMap<String, String>>(),
    }
  }
}

impl From<&CodeGenerationResults> for JsCodegenerationResults {
  fn from(results: &CodeGenerationResults) -> Self {
    let (map, id_result_map) = results.inner();

    Self {
      map: map
        .iter()
        .map(|(module_id, runtime_result_map)| {
          let mut runtime_map: HashMap<String, JsCodegenerationResult> = Default::default();
          match &runtime_result_map.mode {
            rspack_core::RuntimeMode::Empty => {}
            rspack_core::RuntimeMode::SingleEntry => {
              runtime_map.insert(
                get_runtime_key(runtime_result_map.single_runtime.as_ref().expect("exist")).clone(),
                id_result_map
                  .get(&runtime_result_map.single_value.expect("TODO"))
                  .expect("TODO")
                  .as_ref()
                  .into(),
              );
            }
            rspack_core::RuntimeMode::Map => {
              runtime_result_map.map.iter().for_each(|(k, v)| {
                runtime_map.insert(
                  k.clone(),
                  id_result_map.get(v).expect("TODO").as_ref().into(),
                );
              });
            }
          };

          (module_id.to_string(), runtime_map)
        })
        .collect(),
    }
  }
}
