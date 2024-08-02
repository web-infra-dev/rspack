use napi_derive::napi;
use rspack_core::{Compilation, EntryOptions, EntryRuntime};
use rspack_napi::napi::bindgen_prelude::*;

use super::dependency::JsDependency;
use crate::library::JsLibraryOptions;

#[napi(object)]
pub struct JsEntryOptions {
  pub name: Option<String>,
  #[napi(ts_type = "false | string")]
  pub runtime: Option<Either<bool, String>>,
  pub chunk_loading: Option<String>,
  pub async_chunks: Option<bool>,
  pub base_uri: Option<String>,
  // TODO
  // public_path
  // filename
  pub library: Option<JsLibraryOptions>,
  pub depend_on: Option<Vec<String>>,
  pub layer: Option<String>,
}

impl From<EntryOptions> for JsEntryOptions {
  fn from(value: EntryOptions) -> Self {
    Self {
      name: value.name,
      runtime: value.runtime.map(|rt| match rt {
        EntryRuntime::String(s) => Either::B(s),
        EntryRuntime::False => Either::A(false),
      }),
      chunk_loading: value.chunk_loading.map(|s| s.into()),
      async_chunks: value.async_chunks,
      base_uri: value.base_uri,
      library: value.library.map(|l| l.into()),
      depend_on: value.depend_on,
      layer: value.layer,
    }
  }
}

#[napi(object)]
pub struct JsEntryData {
  pub dependencies: Vec<ClassInstance<JsDependency>>,
  pub include_dependencies: Vec<ClassInstance<JsDependency>>,
  pub options: JsEntryOptions,
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
          options: e.options.clone().into(),
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
