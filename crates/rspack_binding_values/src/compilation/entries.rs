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
pub struct JsEntries {
  compilation: &'static mut Compilation,
}

impl JsEntries {
  pub fn new(compilation: &'static mut Compilation) -> Self {
    Self { compilation }
  }
}

#[napi]
impl JsEntries {
  #[napi]
  pub fn clear(&mut self) {
    self.compilation.entries.drain(..);
  }

  #[napi(getter)]
  pub fn size(&mut self) -> u32 {
    self.compilation.entries.len() as u32
  }

  #[napi]
  pub fn has(&self, key: String) -> bool {
    self.compilation.entries.contains_key(&key)
  }

  #[napi]
  pub fn set(&mut self, _key: String, _value: JsEntryData) {
    unimplemented!()
  }

  #[napi]
  pub fn delete(&mut self, key: String) -> bool {
    let r = self.compilation.entries.swap_remove(&key);
    match r {
      Some(_) => true,
      None => false,
    }
  }

  #[napi]
  pub fn get(&'static self, env: Env, key: String) -> Result<Either<JsEntryData, ()>> {
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
        Either::A(JsEntryData {
          dependencies,
          include_dependencies,
        })
      }
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn keys(&self) -> Vec<&String> {
    self.compilation.entries.keys().collect()
  }
}
