use napi::sys::napi_value;
use napi_derive::napi;
use rspack_core::{ChunkLoading, EntryData, EntryOptions, EntryRuntime, Root};
use rspack_napi::napi::bindgen_prelude::*;

use crate::{
  dependency::Dependency, entry::JsEntryOptions, library::JsLibraryOptions, DependencyWrapper,
  JsCompilation, RawChunkLoading, WithFalse,
};

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
  pub fn name(&self) -> Either<&String, ()> {
    self.0.name.as_ref().into()
  }

  #[napi(setter)]
  pub fn set_name(&mut self, name: Either<String, ()>) {
    self.0.name = match name {
      Either::A(s) => Some(s),
      Either::B(_) => None,
    };
  }

  #[napi(getter, ts_return_type = "false | string | undefined")]
  pub fn runtime(&self) -> Either3<bool, &String, ()> {
    match &self.0.runtime {
      Some(rt) => match rt {
        EntryRuntime::String(s) => Either3::B(s),
        EntryRuntime::False => Either3::A(false),
      },
      None => Either3::C(()),
    }
  }

  #[napi(setter)]
  pub fn set_runtime(&mut self, chunk_loading: Either3<bool, String, ()>) {
    self.0.chunk_loading = match chunk_loading {
      Either3::A(_) => Some(ChunkLoading::Disable),
      Either3::B(s) => Some(ChunkLoading::Enable(s.as_str().into())),
      Either3::C(_) => None,
    };
  }

  #[napi(getter)]
  pub fn chunk_loading(&self) -> Either<&str, ()> {
    match &self.0.chunk_loading {
      Some(c) => Either::A(c.as_str()),
      None => Either::B(()),
    }
  }

  #[napi(setter, ts_type = "(chunkLoading: string | false | undefined)")]
  pub fn set_chunk_loading(&mut self, chunk_loading: Either<RawChunkLoading, ()>) {
    match chunk_loading {
      Either::A(WithFalse::False) => self.0.chunk_loading = Some(ChunkLoading::Disable),
      Either::A(WithFalse::True(s)) => {
        self.0.chunk_loading = Some(ChunkLoading::Enable(s.as_str().into()))
      }
      Either::B(_) => self.0.chunk_loading = None,
    }
  }

  #[napi(getter)]
  pub fn async_chunks(&self) -> Either<bool, ()> {
    self.0.async_chunks.into()
  }

  #[napi(setter)]
  pub fn set_async_chunks(&mut self, async_chunks: Either<bool, ()>) {
    self.0.async_chunks = match async_chunks {
      Either::A(b) => Some(b),
      Either::B(_) => None,
    };
  }

  #[napi(getter)]
  pub fn base_uri(&self) -> Either<&String, ()> {
    self.0.base_uri.as_ref().into()
  }

  #[napi(setter)]
  pub fn set_base_uri(&mut self, base_uri: Either<String, ()>) {
    self.0.base_uri = match base_uri {
      Either::A(s) => Some(s),
      Either::B(_) => None,
    };
  }

  #[napi(getter)]
  pub fn library(&self) -> Either<JsLibraryOptions, ()> {
    self.0.library.clone().map(Into::into).into()
  }

  #[napi(setter)]
  pub fn set_library(&mut self, library: Either<JsLibraryOptions, ()>) {
    self.0.library = match library {
      Either::A(l) => Some(l.into()),
      Either::B(_) => None,
    };
  }

  #[napi(getter)]
  pub fn depend_on(&self) -> Either<&Vec<String>, ()> {
    self.0.depend_on.as_ref().into()
  }

  #[napi(setter)]
  pub fn set_depend_on(&mut self, depend_on: Either<Vec<String>, ()>) {
    self.0.depend_on = match depend_on {
      Either::A(vec) => Some(vec),
      Either::B(_) => None,
    };
  }

  #[napi(getter)]
  pub fn layer(&self) -> Either<&String, ()> {
    self.0.layer.as_ref().into()
  }

  #[napi(setter)]
  pub fn set_layer(&mut self, layer: Either<String, ()>) {
    self.0.layer = match layer {
      Either::A(s) => Some(s),
      Either::B(_) => None,
    };
  }

  // #[napi(getter)]
  // pub fn public_path(&self) -> Either3<String, JsFunction, ()> {
  //   unimplemented!()
  // }

  // #[napi(setter)]
  // pub fn set_public_path(&self, _public_path: Option<Either<String, JsFunction>>) {
  //   unimplemented!()
  // }

  // #[napi(getter)]
  // pub fn filename(&self) -> Either3<String, JsFunction, ()> {
  //   unimplemented!()
  // }

  // #[napi(setter)]
  // pub fn set_filename(&self, _filename: Option<Either<String, JsFunction>>) {
  //   unimplemented!()
  // }
}

#[napi(object, object_to_js = false)]
pub struct JsEntryData {
  pub dependencies: Vec<ClassInstance<'static, Dependency>>,
  pub include_dependencies: Vec<ClassInstance<'static, Dependency>>,
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
  i: Box<rspack_core::EntryData>,
  // The WeakReference<JsCompilation> is inserted in the binding entries.get and entries.values methods
  pub(crate) compilation: Option<WeakReference<JsCompilation>>,
}

impl EntryDataDTO {
  pub fn new(entry_data: Box<rspack_core::EntryData>) -> Self {
    Self {
      i: entry_data,
      compilation: None,
    }
  }
}

#[napi]
impl EntryDataDTO {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&mut self) -> Vec<DependencyWrapper> {
    let Some(weak_reference) = &mut self.compilation else {
      return vec![];
    };
    let Some(js_compilation) = weak_reference.get_mut() else {
      return vec![];
    };
    let compilation = &mut *js_compilation.0;
    let module_graph = compilation.get_module_graph();
    self
      .i
      .dependencies
      .iter()
      .map(|dependency_id| {
        #[allow(clippy::unwrap_used)]
        let dep = module_graph.dependency_by_id(dependency_id).unwrap();
        DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation))
      })
      .collect::<Vec<_>>()
  }

  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn include_dependencies(&mut self) -> Vec<DependencyWrapper> {
    let Some(weak_reference) = &mut self.compilation else {
      return vec![];
    };
    let Some(js_compilation) = weak_reference.get_mut() else {
      return vec![];
    };
    let compilation = &mut *js_compilation.0;
    let module_graph = compilation.get_module_graph();
    self
      .i
      .include_dependencies
      .iter()
      .map(|dependency_id| {
        #[allow(clippy::unwrap_used)]
        let dep = module_graph.dependency_by_id(dependency_id).unwrap();
        DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation))
      })
      .collect::<Vec<_>>()
  }

  #[napi(getter)]
  pub fn options<'scope>(
    &self,
    env: &'scope Env,
  ) -> Result<ClassInstance<'scope, EntryOptionsDTO>> {
    EntryOptionsDTO::new(self.i.options.clone()).into_instance(env)
  }
}

#[napi]
pub struct JsEntries {
  i: Box<rspack_core::Entries>,
  // The WeakReference<JsCompilation> is inserted in the binding compilation.entries method
  pub(crate) compilation: Option<WeakReference<JsCompilation>>,
}

impl JsEntries {
  pub fn new(entries: Box<rspack_core::Entries>) -> Self {
    Self {
      i: entries,
      compilation: None,
    }
  }
}

#[napi]
impl JsEntries {
  #[napi]
  pub fn clear(&mut self) {
    self.i.drain(..);
  }

  #[napi(getter)]
  pub fn size(&mut self) -> u32 {
    self.i.len() as u32
  }

  #[napi]
  pub fn has(&self, key: String) -> bool {
    self.i.contains_key(&key)
  }

  #[napi]
  pub fn set(&mut self, key: String, entry_data_object: JsEntryData) {
    let entry_data = Root::new(entry_data_object.into());
    self.i.insert(key, entry_data);
  }

  #[napi]
  pub fn delete(&mut self, key: String) -> bool {
    let r = self.i.swap_remove(&key);
    r.is_some()
  }

  #[napi(ts_return_type = "JsEntryData")]
  pub fn get(&mut self, env: Env, key: String) -> Result<Either<napi_value, ()>> {
    Ok(match self.i.get_mut(&key) {
      Some(entry_data) => unsafe {
        let napi_value = ToNapiValue::to_napi_value(env.raw(), entry_data)?;
        let js_entry_data = JsEntries::from_napi_mut_ref(env.raw(), napi_value)?;
        js_entry_data.compilation = self.compilation.clone();
        Either::A(napi_value)
      },
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn keys(&self) -> Vec<&String> {
    self.i.keys().collect()
  }

  #[napi(ts_return_type = "JsEntryData[]")]
  pub fn values(&mut self, env: Env) -> napi::Result<Vec<napi_value>> {
    self
      .i
      .values_mut()
      .map(|entry_data| unsafe {
        let napi_value = ToNapiValue::to_napi_value(env.raw(), entry_data)?;
        let js_entry_data = JsEntries::from_napi_mut_ref(env.raw(), napi_value)?;
        js_entry_data.compilation = self.compilation.clone();
        Ok(napi_value)
      })
      .collect()
  }
}
