use std::{cell::RefCell, sync::Weak};

use napi::JsString;
use napi_derive::napi;
use rspack_core::{BindingCell, ChunkLoading, Compilation, EntryData, EntryRuntime};
use rspack_napi::napi::bindgen_prelude::*;

use super::JsCompilation;
use crate::{
  dependency::Dependency, entry::JsEntryOptions, library::JsLibraryOptions, DependencyWrapper,
  JsCompiler, RawChunkLoading, WithFalse,
};

#[napi]
pub struct EntryOptionsDTO {
  pub(crate) i: Weak<RefCell<Box<rspack_core::EntryOptions>>>,
}

impl EntryOptionsDTO {
  fn with_ref<T, F>(&self, f: F) -> napi::Result<T>
  where
    F: FnOnce(&rspack_core::EntryOptions) -> napi::Result<T>,
  {
    match self.i.upgrade() {
      Some(entry_options) => f(&**entry_options.borrow()),
      None => Err(napi::Error::from_reason(
        "Unable to access EntryOptions now. The EntryOptions has been dropped by Rust.",
      )),
    }
  }

  pub(crate) fn with_mut<T, F>(&self, f: F) -> napi::Result<T>
  where
    F: FnOnce(&mut rspack_core::EntryOptions) -> napi::Result<T>,
  {
    match self.i.upgrade() {
      Some(entry_options) => f(&mut **entry_options.borrow_mut()),
      None => Err(napi::Error::from_reason(
        "Unable to access EntryOptions now. The EntryOptions has been dropped by Rust.",
      )),
    }
  }
}

#[napi]
impl EntryOptionsDTO {
  #[napi(getter)]
  pub fn name(&self, env: &Env) -> napi::Result<Either<JsString, ()>> {
    self.with_ref(|entry_options| {
      Ok(match &entry_options.name {
        Some(name) => {
          let js_name = env.create_string(name)?;
          Either::A(js_name)
        }
        None => Either::B(()),
      })
    })
  }

  #[napi(setter)]
  pub fn set_name(&mut self, name: Either<String, ()>) -> napi::Result<()> {
    self.with_mut(|entry_options| {
      entry_options.name = match name {
        Either::A(s) => Some(s),
        Either::B(_) => None,
      };
      Ok(())
    })
  }

  #[napi(getter, ts_return_type = "false | string | undefined")]
  pub fn runtime(&self) -> napi::Result<Either3<bool, &String, ()>> {
    // match &self.0.runtime {
    //   Some(rt) => match rt {
    //     EntryRuntime::String(s) => Either3::B(s),
    //     EntryRuntime::False => Either3::A(false),
    //   },
    //   None => Either3::C(()),
    // }
    todo!()
  }

  #[napi(setter)]
  pub fn set_runtime(&mut self, chunk_loading: Either3<bool, String, ()>) {
    // self.0.chunk_loading = match chunk_loading {
    //   Either3::A(_) => Some(ChunkLoading::Disable),
    //   Either3::B(s) => Some(ChunkLoading::Enable(s.as_str().into())),
    //   Either3::C(_) => None,
    // };
    todo!()
  }

  #[napi(getter)]
  pub fn chunk_loading(&self) -> Either<&str, ()> {
    // match &self.0.chunk_loading {
    //   Some(c) => Either::A(c.as_str()),
    //   None => Either::B(()),
    // }
    todo!()
  }

  #[napi(setter, ts_type = "(chunkLoading: string | false | undefined)")]
  pub fn set_chunk_loading(&mut self, chunk_loading: Either<RawChunkLoading, ()>) {
    // match chunk_loading {
    //   Either::A(WithFalse::False) => self.0.chunk_loading = Some(ChunkLoading::Disable),
    //   Either::A(WithFalse::True(s)) => {
    //     self.0.chunk_loading = Some(ChunkLoading::Enable(s.as_str().into()))
    //   }
    //   Either::B(_) => self.0.chunk_loading = None,
    // }
    todo!()
  }

  #[napi(getter)]
  pub fn async_chunks(&self) -> Either<bool, ()> {
    // self.0.async_chunks.into()
    todo!()
  }

  #[napi(setter)]
  pub fn set_async_chunks(&mut self, async_chunks: Either<bool, ()>) {
    // self.0.async_chunks = match async_chunks {
    //   Either::A(b) => Some(b),
    //   Either::B(_) => None,
    // };
    todo!()
  }

  #[napi(getter)]
  pub fn base_uri(&self) -> Either<&String, ()> {
    // self.0.base_uri.as_ref().into()
    todo!()
  }

  #[napi(setter)]
  pub fn set_base_uri(&mut self, base_uri: Either<String, ()>) {
    // self.0.base_uri = match base_uri {
    //   Either::A(s) => Some(s),
    //   Either::B(_) => None,
    // };
    todo!()
  }

  #[napi(getter)]
  pub fn library(&self) -> Either<JsLibraryOptions, ()> {
    // self.0.library.clone().map(Into::into).into()
    todo!()
  }

  #[napi(setter)]
  pub fn set_library(&mut self, library: Either<JsLibraryOptions, ()>) {
    // self.0.library = match library {
    //   Either::A(l) => Some(l.into()),
    //   Either::B(_) => None,
    // };
    todo!()
  }

  #[napi(getter)]
  pub fn depend_on(&self) -> Either<&Vec<String>, ()> {
    // self.0.depend_on.as_ref().into()
    todo!()
  }

  #[napi(setter)]
  pub fn set_depend_on(&mut self, depend_on: Either<Vec<String>, ()>) {
    // self.0.depend_on = match depend_on {
    //   Either::A(vec) => Some(vec),
    //   Either::B(_) => None,
    // };
    todo!()
  }

  #[napi(getter)]
  pub fn layer(&self) -> Either<&String, ()> {
    // self.0.layer.as_ref().into()
    todo!()
  }

  #[napi(setter)]
  pub fn set_layer(&mut self, layer: Either<String, ()>) {
    // self.0.layer = match layer {
    //   Either::A(s) => Some(s),
    //   Either::B(_) => None,
    // };
    todo!()
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
    let options = rspack_core::EntryOptions::from(value.options);

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
      options: BindingCell::from(options),
    }
  }
}

#[napi]
pub struct EntryDataDTO {
  pub(crate) i: Weak<RefCell<Box<rspack_core::EntryData>>>,
  pub(crate) compiler_reference: Option<WeakReference<JsCompiler>>,
}

impl EntryDataDTO {
  fn with_ref<T, F>(&self, f: F) -> napi::Result<T>
  where
    F: FnOnce(&Compilation, &rspack_core::EntryData) -> napi::Result<T>,
  {
    let Some(compiler_reference) = &self.compiler_reference else {
      return Err(napi::Error::from_reason(
        "Not binding the compiler reference.",
      ));
    };
    match compiler_reference.get() {
      Some(this) => match self.i.upgrade() {
        Some(entry_data) => f(&this.compiler.compilation, &**entry_data.borrow()),
        None => Err(napi::Error::from_reason(
          "Unable to access EntryData now. The EntryData has been dropped by Rust.",
        )),
      },
      None => Err(napi::Error::from_reason(
        "Unable to access EntryData now. The Compiler has been garbage collected by JavaScript.",
      )),
    }
  }
}

#[napi]
impl EntryDataDTO {
  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn dependencies(&self) -> napi::Result<Vec<DependencyWrapper>> {
    self.with_ref(|compilation, entry_data| {
      let module_graph = compilation.get_module_graph();
      Ok(
        entry_data
          .dependencies
          .iter()
          .map(|dependency_id| {
            #[allow(clippy::unwrap_used)]
            let dep = module_graph.dependency_by_id(dependency_id).unwrap();
            DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation))
          })
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(getter, ts_return_type = "Dependency[]")]
  pub fn include_dependencies(&self) -> napi::Result<Vec<DependencyWrapper>> {
    self.with_ref(|compilation, entry_data| {
      let module_graph = compilation.get_module_graph();
      Ok(
        entry_data
          .include_dependencies
          .iter()
          .map(|dependency_id| {
            #[allow(clippy::unwrap_used)]
            let dep = module_graph.dependency_by_id(dependency_id).unwrap();
            DependencyWrapper::new(dep.as_ref(), compilation.id(), Some(compilation))
          })
          .collect::<Vec<_>>(),
      )
    })
  }

  #[napi(getter)]
  pub fn options(&self, env: &Env, mut this: This) -> napi::Result<ClassInstance<EntryOptionsDTO>> {
    self.with_ref(|compilation, entry_data| {
      entry_data.options.to_jsobject(env, &mut this.object)?;
      // EntryOptionsDTO::new(self.entry_data.options.clone()).into_instance(env)
      todo!()
    })
  }
}

#[napi]
pub struct JsEntries {
  compiler_reference: WeakReference<JsCompiler>,
}

impl JsEntries {
  pub fn new(compiler_reference: WeakReference<JsCompiler>) -> Self {
    Self { compiler_reference }
  }

  fn with_ref<T, F>(&self, f: F) -> napi::Result<T>
  where
    F: FnOnce(&rspack_core::Entries) -> napi::Result<T>,
  {
    match self.compiler_reference.get() {
      Some(reference) => f(&reference.compiler.compilation.entries),
      None => Err(napi::Error::from_reason(
        "Unable to access entries now. The Compilation has been garbage collected by JavaScript.",
      )),
    }
  }

  fn with_mut<T, F>(&mut self, f: F) -> napi::Result<T>
  where
    F: FnOnce(&mut rspack_core::Entries) -> napi::Result<T>,
  {
    match self.compiler_reference.get_mut() {
      Some(reference) => f(&mut reference.compiler.compilation.entries),
      None => Err(napi::Error::from_reason(
        "Unable to access entries now. The Compilation has been garbage collected by JavaScript.",
      )),
    }
  }
}

#[napi]
impl JsEntries {
  #[napi]
  pub fn clear(&mut self) -> napi::Result<()> {
    self.with_mut(|entries| {
      entries.clear();
      Ok(())
    })
  }

  #[napi(getter)]
  pub fn size(&self) -> napi::Result<u32> {
    self.with_ref(|entries| Ok(entries.len() as u32))
  }

  #[napi]
  pub fn has(&self, key: String) -> napi::Result<bool> {
    self.with_ref(|entries| Ok(entries.contains_key(&key)))
  }

  #[napi]
  pub fn set(&mut self, key: String, value: JsEntryData) {
    // let entry_data = match value {
    //   Either::A(js) => js.into(),
    //   Either::B(dto) => {
    //     assert!(
    //       std::ptr::eq(dto.compilation, self.compilation),
    //       "The set() method cannot accept entry data from a different compilation instance."
    //     );
    //     dto.entry_data.clone()
    //   }
    // };
    // self.compilation.entries.insert(key, entry_data);
    todo!()
  }

  #[napi]
  pub fn delete(&mut self, key: String) -> napi::Result<bool> {
    self.with_mut(|entries| {
      let r = entries.swap_remove(&key);
      Ok(r.is_some())
    })
  }

  #[napi(ts_return_type = "EntryDataDTO | undefined")]
  pub fn get(&self, env: &Env, mut this: This, key: String) -> Result<Either<Object, ()>> {
    self.with_ref(|entries| {
      Ok(match entries.get(&key) {
        Some(entry_data) => Either::A(entry_data.to_jsobject(env, &mut this.object)?),
        None => Either::B(()),
      })
    })
  }

  #[napi(ts_return_type = "string[]")]
  pub fn keys(&self, env: &Env) -> napi::Result<Array> {
    self.with_ref(|entries| {
      let keys = entries.keys();
      let mut array = env.create_array(0)?;
      for key in keys {
        array.insert(env.create_string(key)?)?;
      }
      Ok(array)
    })
  }

  #[napi]
  pub fn values(&self, env: &Env, mut this: This) -> napi::Result<Array> {
    self.with_ref(|entries| {
      let mut array = env.create_array(0)?;
      for value in entries.values() {
        let object = value.to_jsobject(env, &mut this.object)?;
        array.insert(object)?;
      }
      Ok(array)
    })
  }
}
