use std::ptr::NonNull;

use napi_derive::napi;
use rspack_core::{ChunkLoading, Compilation, EntryData, EntryOptions, EntryRuntime};
use rspack_napi::napi::bindgen_prelude::*;

use crate::{
  dependency::JsDependency, entry::JsEntryOptions, library::JsLibraryOptions, JsDependencyWrapper,
  RawChunkLoading, WithFalse,
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
      Some(c) => Either::A(c.into()),
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
pub struct RawEntryData {
  pub dependencies: Vec<ClassInstance<'static, JsDependency>>,
  pub include_dependencies: Vec<ClassInstance<'static, JsDependency>>,
  pub options: JsEntryOptions,
}

impl From<RawEntryData> for EntryData {
  fn from(value: RawEntryData) -> Self {
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
pub struct JsEntryData {
  compilation: NonNull<Compilation>,
  key: String,
}

impl JsEntryData {
  fn as_ref(&self) -> napi::Result<(&'static EntryData, &'static Compilation)> {
    let compilation = unsafe { self.compilation.as_ref() };
    if let Some(entry_data) = compilation.entries.get(&self.key) {
      Ok((entry_data, compilation))
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access EntryData with key = {} now. The EntryData have been removed on the Rust side.",
        self.key
      )))
    }
  }

  fn as_mut(&mut self) -> napi::Result<&'static mut EntryData> {
    let compilation = unsafe { self.compilation.as_mut() };
    if let Some(entry_data) = compilation.entries.get_mut(&self.key) {
      Ok(entry_data)
    } else {
      Err(napi::Error::from_reason(format!(
        "Unable to access EntryData with key = {} now. The EntryData have been removed on the Rust side.",
        self.key
      )))
    }
  }
}

#[napi]
impl JsEntryData {
  #[napi(getter, ts_return_type = "JsDependency[]")]
  pub fn dependencies(&self) -> Result<Vec<JsDependencyWrapper>> {
    let (entry_data, compilation) = self.as_ref()?;
    let module_graph = compilation.get_module_graph();
    Ok(
      entry_data
        .dependencies
        .iter()
        .map(|dependency_id| {
          #[allow(clippy::unwrap_used)]
          let dep = module_graph.dependency_by_id(dependency_id).unwrap();
          JsDependencyWrapper::new(
            dep.as_ref(),
            compilation.compiler_id(),
            compilation.id(),
            Some(compilation),
          )
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi(setter)]
  pub fn set_dependencies(
    &mut self,
    dependencies: Vec<ClassInstance<'static, JsDependency>>,
  ) -> Result<()> {
    let entry_data = self.as_mut()?;
    entry_data.dependencies = dependencies
      .into_iter()
      .map(|js_dep| js_dep.dependency_id)
      .collect::<Vec<_>>();
    Ok(())
  }

  #[napi(getter, ts_return_type = "JsDependency[]")]
  pub fn include_dependencies(&'static self) -> Result<Vec<JsDependencyWrapper>> {
    let (entry_data, compilation) = self.as_ref()?;
    let module_graph = compilation.get_module_graph();
    Ok(
      entry_data
        .include_dependencies
        .iter()
        .map(|dependency_id| {
          #[allow(clippy::unwrap_used)]
          let dep = module_graph.dependency_by_id(dependency_id).unwrap();
          JsDependencyWrapper::new(
            dep.as_ref(),
            compilation.compiler_id(),
            compilation.id(),
            Some(compilation),
          )
        })
        .collect::<Vec<_>>(),
    )
  }

  #[napi(setter)]
  pub fn set_include_dependencies(
    &mut self,
    dependencies: Vec<ClassInstance<'static, JsDependency>>,
  ) -> Result<()> {
    let entry_data = self.as_mut()?;
    entry_data.include_dependencies = dependencies
      .into_iter()
      .map(|js_dep| js_dep.dependency_id)
      .collect::<Vec<_>>();
    Ok(())
  }

  #[napi(getter)]
  pub fn options<'scope>(
    &self,
    env: &'scope Env,
  ) -> Result<ClassInstance<'scope, EntryOptionsDTO>> {
    let (entry_data, _) = self.as_ref()?;
    EntryOptionsDTO::new(entry_data.options.clone()).into_instance(env)
  }
}

#[napi]
pub struct JsEntries {
  compilation: NonNull<Compilation>,
}

impl JsEntries {
  pub fn new(compilation: &Compilation) -> Self {
    #[allow(clippy::unwrap_used)]
    Self {
      compilation: NonNull::new(compilation as *const Compilation as *mut Compilation).unwrap(),
    }
  }

  fn as_ref(&self) -> Result<&'static Compilation> {
    let compilation = unsafe { self.compilation.as_ref() };
    Ok(compilation)
  }

  fn as_mut(&mut self) -> Result<&'static mut Compilation> {
    let compilation = unsafe { self.compilation.as_mut() };
    Ok(compilation)
  }
}

#[napi]
impl JsEntries {
  #[napi]
  pub fn clear(&mut self) -> Result<()> {
    let compilation = self.as_mut()?;
    compilation.entries.drain(..);
    Ok(())
  }

  #[napi(getter)]
  pub fn size(&mut self) -> Result<u32> {
    let compilation = self.as_ref()?;
    Ok(compilation.entries.len() as u32)
  }

  #[napi]
  pub fn has(&self, key: String) -> Result<bool> {
    let compilation = self.as_ref()?;
    Ok(compilation.entries.contains_key(&key))
  }

  #[napi]
  pub fn set(&mut self, key: String, value: RawEntryData) -> Result<()> {
    let compilation = self.as_mut()?;
    compilation.entries.insert(key, value.into());
    Ok(())
  }

  #[napi]
  pub fn delete(&mut self, key: String) -> Result<bool> {
    let compilation = self.as_mut()?;
    let r = compilation.entries.swap_remove(&key);
    Ok(r.is_some())
  }

  #[napi]
  pub fn get(&self, key: String) -> Result<Either<JsEntryData, ()>> {
    let compilation = self.as_ref()?;
    Ok(match compilation.entries.get(&key) {
      Some(_) => Either::A(JsEntryData {
        key,
        compilation: self.compilation,
      }),
      None => Either::B(()),
    })
  }

  #[napi]
  pub fn keys(&self) -> Result<Vec<&String>> {
    let compilation = self.as_ref()?;
    Ok(compilation.entries.keys().collect())
  }

  #[napi]
  pub fn values(&self) -> Result<Vec<JsEntryData>> {
    let compilation = self.as_ref()?;
    Ok(
      compilation
        .entries
        .keys()
        .cloned()
        .map(|value| JsEntryData {
          key: value,
          compilation: self.compilation,
        })
        .collect(),
    )
  }
}
