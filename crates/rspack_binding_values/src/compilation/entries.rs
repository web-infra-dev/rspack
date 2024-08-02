use napi_derive::napi;
use rspack_core::{ChunkLoading, Compilation, EntryData, EntryOptions, EntryRuntime};
use rspack_napi::napi::bindgen_prelude::*;

use super::dependency::JsDependency;
use crate::{entry::JsEntryOptions, library::JsLibraryOptions};

#[napi]
pub struct EntryOptionsDTO(EntryOptions);

impl EntryOptionsDTO {
  pub fn new(options: EntryOptions) -> Self {
    Self(options)
  }
}

#[napi]
impl EntryOptionsDTO {
  #[napi(getter)]
  pub fn name(&self) -> Option<&String> {
    self.0.name.as_ref()
  }

  #[napi(setter)]
  pub fn set_name(&mut self, name: Option<String>) {
    self.0.name = name;
  }

  #[napi(getter)]
  pub fn runtime(&self) -> Option<Either<bool, &String>> {
    self.0.runtime.as_ref().map(|rt| match rt {
      EntryRuntime::String(s) => Either::B(s),
      EntryRuntime::False => Either::A(false),
    })
  }

  #[napi(setter)]
  pub fn set_runtime(&mut self, chunk_loading: Option<Either<bool, String>>) {
    self.0.chunk_loading = chunk_loading.map(|c| match c {
      Either::A(_) => ChunkLoading::Disable,
      Either::B(s) => ChunkLoading::Enable(s.as_str().into()),
    });
  }

  #[napi(getter)]
  pub fn chunk_loading(&self) -> Option<&str> {
    self.0.chunk_loading.as_ref().map(Into::into)
  }

  #[napi(setter)]
  pub fn set_chunk_loading(&mut self, chunk_loading: Option<String>) {
    self.0.chunk_loading = chunk_loading.map(|s| Into::into(s.as_str()));
  }

  #[napi(getter)]
  pub fn async_chunks(&self) -> Option<bool> {
    self.0.async_chunks
  }

  #[napi(setter)]
  pub fn set_async_chunks(&mut self, async_chunks: Option<bool>) {
    self.0.async_chunks = async_chunks;
  }

  #[napi(getter)]
  pub fn base_uri(&self) -> Option<&String> {
    self.0.base_uri.as_ref()
  }

  #[napi(setter)]
  pub fn set_base_uri(&mut self, base_uri: Option<String>) {
    self.0.base_uri = base_uri;
  }

  #[napi(getter)]
  pub fn library(&self) -> Option<JsLibraryOptions> {
    self.0.library.clone().map(Into::into)
  }

  #[napi(setter)]
  pub fn set_library(&mut self, options: Option<JsLibraryOptions>) {
    self.0.library = options.map(Into::into);
  }

  #[napi(getter)]
  pub fn depend_on(&self) -> Option<&Vec<String>> {
    self.0.depend_on.as_ref()
  }

  #[napi(setter)]
  pub fn set_depend_on(&mut self, depend_on: Option<Vec<String>>) {
    self.0.depend_on = depend_on;
  }

  #[napi(getter)]
  pub fn layer(&self) -> Option<&String> {
    self.0.layer.as_ref()
  }

  #[napi(setter)]
  pub fn set_layer(&mut self, layer: Option<String>) {
    self.0.layer = layer;
  }

  // #[napi(getter)]
  // pub fn public_path(&self) -> Option<Either<String, JsFunction>> {
  //   unimplemented!()
  // }

  // #[napi(getter)]
  // pub fn filename(&self) -> Option<Either<String, JsFunction>> {
  //   unimplemented!()
  // }
}

#[napi(object, object_to_js = false)]
pub struct JsEntryData {
  pub dependencies: Vec<ClassInstance<JsDependency>>,
  pub include_dependencies: Vec<ClassInstance<JsDependency>>,
  pub options: JsEntryOptions,
}

impl From<JsEntryData> for EntryData {
  fn from(value: JsEntryData) -> Self {
    Self {
      dependencies: value
        .dependencies
        .into_iter()
        .map(|dep| dep.dependency_id)
        .collect::<Vec<_>>(),
      include_dependencies: value
        .include_dependencies
        .into_iter()
        .map(|dep| dep.dependency_id)
        .collect::<Vec<_>>(),
      options: value.options.into(),
    }
  }
}

#[napi]
pub struct EntryDataDTO {
  compilation: &'static mut Compilation,
  entry_data: EntryData,
}

#[napi]
impl EntryDataDTO {
  #[napi(getter)]
  pub fn dependencies(&'static self, env: Env) -> Result<Vec<ClassInstance<JsDependency>>> {
    self
      .entry_data
      .dependencies
      .clone()
      .into_iter()
      .map(|id| {
        let js_dep = JsDependency::new(id, self.compilation);
        let instance = js_dep.into_instance(env)?;
        Ok(instance)
      })
      .collect::<Result<Vec<ClassInstance<JsDependency>>>>()
  }

  #[napi(getter)]
  pub fn include_dependencies(&'static self, env: Env) -> Result<Vec<ClassInstance<JsDependency>>> {
    self
      .entry_data
      .include_dependencies
      .clone()
      .into_iter()
      .map(|id| {
        let js_dep = JsDependency::new(id, self.compilation);
        let instance = js_dep.into_instance(env)?;
        Ok(instance)
      })
      .collect::<Result<Vec<ClassInstance<JsDependency>>>>()
  }

  #[napi(getter)]
  pub fn options(&self, env: Env) -> Result<ClassInstance<EntryOptionsDTO>> {
    EntryOptionsDTO::new(self.entry_data.options.clone()).into_instance(env)
  }
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
  pub fn set(&mut self, key: String, value: Either<JsEntryData, ClassInstance<EntryDataDTO>>) {
    let entry_data = match value {
      Either::A(js) => js.into(),
      Either::B(dto) => {
        assert!(
          std::ptr::eq(dto.compilation, self.compilation),
          "The set() method cannot accept entry data from a different compilation instance."
        );
        dto.entry_data.clone()
      }
    };
    self.compilation.entries.insert(key, entry_data);
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
  pub fn get(&'static mut self, key: String) -> Result<Either<EntryDataDTO, ()>> {
    Ok(match self.compilation.entries.get(&key) {
      Some(entry_data) => Either::A(EntryDataDTO {
        entry_data: entry_data.clone(),
        compilation: self.compilation,
      }),
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn keys(&self) -> Vec<&String> {
    self.compilation.entries.keys().collect()
  }
}
