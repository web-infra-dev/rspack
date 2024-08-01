use napi_derive::napi;
use rspack_core::Compilation;
use rspack_napi::napi::bindgen_prelude::*;

use super::dependency::JsDependency;

#[napi(object)]
pub struct JsEntryData {
  pub dependencies: Vec<ClassInstance<JsDependency>>,
  pub include_dependencies: Vec<ClassInstance<JsDependency>>,
  // pub options: JsEntryOptions,
}

#[napi]
pub struct JsEntryDataMap {
  compilation: &'static mut Compilation,
}

impl JsEntryDataMap {
  pub fn new(compilation: &'static mut Compilation) -> Self {
    Self { compilation }
  }
}

#[napi]
impl JsEntryDataMap {
  #[napi]
  pub fn has(&self, key: String) -> bool {
    self.compilation.entries.contains_key(&key)
  }

  pub fn set(&mut self, key: String, value: JsEntryData) {
    unimplemented!()
  }

  #[napi]
  pub fn delete(&mut self, key: String) {
    self.compilation.entries.swap_remove(&key);
  }

  pub fn get(&'static self, env: Env, key: String) -> Result<Option<JsEntryData>> {
    let entry = self.compilation.entries.get(&key);

    Ok(match entry {
      Some(e) => {
        let dependencies = e
          .dependencies
          .clone()
          .into_iter()
          .map(|id| {
            let js_dep = JsDependency::new(id, self.compilation);
            let instance = js_dep.into_instance(env)?;
            Ok(instance)
          })
          .collect::<Result<Vec<ClassInstance<JsDependency>>>>()?;
        let include_dependencies = e
          .include_dependencies
          .clone()
          .into_iter()
          .map(|id| {
            let js_dep = JsDependency::new(id, self.compilation);
            let instance = js_dep.into_instance(env)?;
            Ok(instance)
          })
          .collect::<Result<Vec<ClassInstance<JsDependency>>>>()?;
        Some(JsEntryData {
          dependencies,
          include_dependencies,
        })
      }
      None => None,
    })
  }

  #[napi]
  pub fn keys(&self) -> Vec<&String> {
    self.compilation.entries.keys().collect()
  }
}
