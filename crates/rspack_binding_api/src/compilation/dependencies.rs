use napi::{
  Env, JsString,
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilationId, FileCounter};
use rspack_napi::OneShotRef;
use rspack_paths::{InternedPath, InternedPathIndexSet, InternedPathMap};
use rustc_hash::FxHashSet;

use crate::{with_compilation, with_compilation_mut};

/// Native operations on one compilation dependency collection.
#[napi]
pub struct FileSystemDependencies {
  kind: FileSystemDependencyKind,
  compilation_id: CompilationId,
  // Cached JS values live with this wrapper and are released on GC.
  dependency_strings: Option<DependencyStrings>,
}

struct DependencyStrings {
  // Updated in place: every values() call returns this same array.
  array: OneShotRef,
  indices: InternedPathMap<u32>,
}

fn referenced_array<'env>(env: &'env Env, reference: &OneShotRef) -> napi::Result<Array<'env>> {
  // SAFETY: The reference owns an Array created in this JS environment.
  // Resolve it inside the current handle scope; never retain a napi_value.
  unsafe { Array::from_napi_value(env.raw(), ToNapiValue::to_napi_value(env.raw(), reference)?) }
}

fn create_array<'env>(env: &'env Env, length: usize) -> napi::Result<Array<'env>> {
  // SAFETY: The freshly created JS object is an Array in this handle scope.
  unsafe {
    Array::from_napi_value(
      env.raw(),
      ToNapiValue::to_napi_value(env.raw(), env.create_array_with_length(length)?)?,
    )
  }
}

pub enum FileSystemDependencyKind {
  File,
  Context,
  Missing,
  Build,
}

impl FileSystemDependencies {
  pub fn new(kind: FileSystemDependencyKind, compilation_id: CompilationId) -> Self {
    Self {
      kind,
      compilation_id,
      dependency_strings: None,
    }
  }

  fn dependency_strings(&mut self, env: &Env) -> napi::Result<&mut DependencyStrings> {
    match &mut self.dependency_strings {
      Some(dependency_strings) => Ok(dependency_strings),
      slot @ None => {
        let array = create_array(env, 0)?;
        Ok(slot.insert(DependencyStrings {
          array: OneShotRef::new(env.raw(), array)?,
          indices: InternedPathMap::default(),
        }))
      }
    }
  }

  fn add_values<'env>(
    &mut self,
    env: &Env,
    values: impl IntoIterator<Item = JsString<'env>>,
  ) -> napi::Result<()> {
    with_compilation_mut(self.compilation_id, |compilation| {
      let additions = self.additions(compilation);
      let dependency_strings = self.dependency_strings(env)?;
      let mut array = referenced_array(env, &dependency_strings.array)?;
      for value in values {
        let utf8 = value.into_utf8()?;
        let path: InternedPath = utf8.as_str()?.into();
        additions.insert(path.clone());
        if !dependency_strings.indices.contains_key(&path) {
          let index = array.len();
          // Retain the original JS string without reading the in-progress
          // module graph or copying the existing array.
          array.set(index, value)?;
          dependency_strings.indices.insert(path, index);
        }
      }
      Ok(())
    })
  }

  fn sources<'a>(
    &self,
    compilation: &'a Compilation,
  ) -> napi::Result<(&'a FileCounter, &'a InternedPathIndexSet)> {
    if compilation.build_module_graph_artifact.is_stolen() {
      return Err(napi::Error::from_reason(
        "Compilation dependencies are not available while a compilation pass is holding the module graph artifact",
      ));
    }
    let artifact = &compilation.build_module_graph_artifact;
    let (counter, added) = match self.kind {
      FileSystemDependencyKind::File => {
        (&artifact.file_dependencies, &compilation.file_dependencies)
      }
      FileSystemDependencyKind::Context => (
        &artifact.context_dependencies,
        &compilation.context_dependencies,
      ),
      FileSystemDependencyKind::Missing => (
        &artifact.missing_dependencies,
        &compilation.missing_dependencies,
      ),
      FileSystemDependencyKind::Build => (
        &artifact.build_dependencies,
        &compilation.build_dependencies,
      ),
    };
    Ok((counter, added))
  }

  fn paths<'a>(
    &self,
    compilation: &'a Compilation,
  ) -> napi::Result<impl Iterator<Item = &'a InternedPath>> {
    let (counter, added) = self.sources(compilation)?;
    Ok(counter.files().chain(added))
  }

  fn additions<'a>(&self, compilation: &'a mut Compilation) -> &'a mut InternedPathIndexSet {
    match self.kind {
      FileSystemDependencyKind::File => &mut compilation.file_dependencies,
      FileSystemDependencyKind::Context => &mut compilation.context_dependencies,
      FileSystemDependencyKind::Missing => &mut compilation.missing_dependencies,
      FileSystemDependencyKind::Build => &mut compilation.build_dependencies,
    }
  }
}

#[napi]
impl FileSystemDependencies {
  #[napi(getter)]
  pub fn added(&self) -> napi::Result<Vec<String>> {
    with_compilation(self.compilation_id, |compilation| {
      let (counter, added) = self.sources(compilation)?;
      // Plugin additions are included in every build's watch delta.
      Ok(
        counter
          .added_files()
          .chain(added)
          .map(|path| path.to_string_lossy().into_owned())
          .collect(),
      )
    })
  }

  #[napi(getter)]
  pub fn removed(&self) -> napi::Result<Vec<String>> {
    with_compilation(self.compilation_id, |compilation| {
      let (counter, _) = self.sources(compilation)?;
      Ok(
        counter
          .removed_files()
          .map(|path| path.to_string_lossy().into_owned())
          .collect(),
      )
    })
  }

  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_compilation(self.compilation_id, |compilation| {
      // Match values() by deduplicating native paths without converting strings.
      let paths: FxHashSet<_> = self.paths(compilation)?.collect();
      Ok(paths.len() as u32)
    })
  }

  #[napi]
  pub fn has(&self, value: String) -> napi::Result<bool> {
    with_compilation(self.compilation_id, |compilation| {
      Ok(
        self
          .paths(compilation)?
          .any(|path| path.to_string_lossy() == value),
      )
    })
  }

  #[napi(ts_return_type = "ReadonlyArray<string>")]
  pub fn values<'env>(&mut self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_compilation(self.compilation_id, |compilation| {
      let paths = self.paths(compilation)?;
      let capacity = paths.size_hint().0;
      let mut unique_paths = Vec::with_capacity(capacity);
      let mut indices = InternedPathMap::with_capacity_and_hasher(capacity, Default::default());
      for path in paths {
        // A path can occur in both native dependencies and plugin additions.
        indices.entry(path.clone()).or_insert_with(|| {
          let index = unique_paths.len() as u32;
          unique_paths.push(path);
          index
        });
      }
      let dependency_strings = self.dependency_strings(env)?;
      let mut array = referenced_array(env, &dependency_strings.array)?;
      let length = unique_paths.len() as u32;
      let mut updates = Vec::new();
      for (index, path) in unique_paths.into_iter().enumerate() {
        let index = index as u32;
        let previous_index = dependency_strings.indices.get(path).copied();
        if previous_index == Some(index) {
          continue;
        }
        let value = match previous_index {
          Some(previous_index) => array
            .get::<JsString>(previous_index)?
            .ok_or_else(|| napi::Error::from_reason("Missing cached dependency string"))?,
          None => env.create_string(path.to_string_lossy().as_ref())?,
        };
        updates.push((index, value));
      }
      // Resolve all reused strings before writing: moves can overwrite slots
      // that other paths still need. These handles live only in this call.
      for (index, value) in updates {
        array.set(index, value)?;
      }
      if array.len() > length {
        // Truncation releases strings belonging to removed paths.
        array.set_named_property("length", length)?;
        array = referenced_array(env, &dependency_strings.array)?;
      }
      dependency_strings.indices = indices;
      Ok(array)
    })
  }

  #[napi]
  pub fn add(&mut self, env: &Env, value: JsString<'_>) -> napi::Result<()> {
    self.add_values(env, [value])
  }

  #[napi]
  pub fn add_all(&mut self, env: &Env, values: Vec<JsString<'_>>) -> napi::Result<()> {
    self.add_values(env, values)
  }
}
