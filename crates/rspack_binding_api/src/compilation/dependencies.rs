use napi::{
  Env, JsString,
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilerId, FileCounter};
use rspack_napi::OneShotRef;
use rspack_paths::{InternedPath, InternedPathIndexSet, InternedPathMap, InternedPathSet};

use crate::COMPILER_REFERENCES;

/// Native operations on the owning compiler's current dependency collection.
#[napi]
pub struct FileSystemDependencies {
  kind: FileSystemDependencyKind,
  compiler_id: CompilerId,
  deleted_paths: InternedPathSet,
  // Cached JS values live with this wrapper and are released on GC.
  dependency_strings: Option<DependencyStrings>,
}

struct DependencyStrings {
  // Updated in place: every values() call returns this same array.
  array: OneShotRef,
  indices: InternedPathMap<u32>,
}

// Retained JS dependency wrappers follow rebuilds, just like JsCompilation.
// Resolve the weak compiler reference on each access without retaining its graph.
fn with_current_compilation<R>(
  compiler_id: CompilerId,
  f: impl FnOnce(&Compilation) -> napi::Result<R>,
) -> napi::Result<R> {
  let compiler_reference =
    COMPILER_REFERENCES.with(|references| references.borrow().get(&compiler_id).cloned());
  let compiler = compiler_reference
    .as_ref()
    .and_then(|reference| reference.get())
    .ok_or_else(|| napi::Error::from_reason(format!(
      "Unable to access dependencies for compiler with id = {compiler_id:?}. The Compiler has been garbage collected by JavaScript."
    )))?;
  f(&compiler.compiler.compilation)
}

fn with_current_compilation_mut<R>(
  compiler_id: CompilerId,
  f: impl FnOnce(&mut Compilation) -> napi::Result<R>,
) -> napi::Result<R> {
  let mut compiler_reference =
    COMPILER_REFERENCES.with(|references| references.borrow().get(&compiler_id).cloned());
  let compiler = compiler_reference
    .as_mut()
    .and_then(|reference| reference.get_mut())
    .ok_or_else(|| napi::Error::from_reason(format!(
      "Unable to access dependencies for compiler with id = {compiler_id:?}. The Compiler has been garbage collected by JavaScript."
    )))?;
  f(&mut compiler.compiler.compilation)
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
  pub fn new(kind: FileSystemDependencyKind, compiler_id: CompilerId) -> Self {
    Self {
      kind,
      compiler_id,
      dependency_strings: None,
      deleted_paths: InternedPathSet::default(),
    }
  }

  fn dependency_strings<'a>(
    env: &Env,
    slot: &'a mut Option<DependencyStrings>,
  ) -> napi::Result<&'a mut DependencyStrings> {
    match slot {
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
    with_current_compilation_mut(self.compiler_id, |compilation| {
      let additions = self.additions(compilation);
      let dependency_strings = Self::dependency_strings(env, &mut self.dependency_strings)?;
      let mut array = referenced_array(env, &dependency_strings.array)?;
      for value in values {
        let utf8 = value.into_utf8()?;
        let path: InternedPath = utf8.as_str()?.into();
        self.deleted_paths.remove(&path);
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
  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn added<'env>(&mut self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (counter, added) = self.sources(compilation)?;
      // Plugin additions are included in every build's watch delta.
      let paths = counter.added_files().chain(added);
      let mut result = create_array(env, paths.size_hint().0)?;
      let Some(dependency_strings) = self.dependency_strings.as_mut() else {
        // Watch may read only this delta once. Avoid building a second array
        // and an index when no collection read or JS addition needs them yet.
        for (index, path) in paths.enumerate() {
          result.set(
            index as u32,
            env.create_string(path.to_string_lossy().as_ref())?,
          )?;
        }
        return Ok(result);
      };
      let mut strings = referenced_array(env, &dependency_strings.array)?;
      for (index, path) in paths.enumerate() {
        let value = match dependency_strings.indices.get(path) {
          Some(&index) => strings
            .get::<JsString>(index)?
            .ok_or_else(|| napi::Error::from_reason("Missing cached dependency string"))?,
          None => {
            // Native dependencies may have changed since values() was read.
            // Root new strings in the shared array, without rescanning all paths.
            let value = env.create_string(path.to_string_lossy().as_ref())?;
            let index = strings.len();
            strings.set(index, value)?;
            dependency_strings.indices.insert(path.clone(), index);
            value
          }
        };
        result.set(index as u32, value)?;
      }
      Ok(result)
    })
  }

  #[napi(getter)]
  pub fn removed(&self) -> napi::Result<Vec<String>> {
    with_current_compilation(self.compiler_id, |compilation| {
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
    with_current_compilation(self.compiler_id, |compilation| {
      let (counter, added) = self.sources(compilation)?;
      let native_count = if self.deleted_paths.is_empty() {
        counter.files().count()
      } else {
        counter
          .files()
          .filter(|path| !self.deleted_paths.contains(*path))
          .count()
      };
      // Both sources are already unique. Count only plugin paths absent from
      // the native counter, without allocating a temporary union set.
      let added_count = added
        .iter()
        .filter(|path| {
          !self.deleted_paths.contains(*path) && counter.related_resource_ids(path).is_none()
        })
        .count();
      Ok((native_count + added_count) as u32)
    })
  }

  #[napi]
  pub fn has(&self, value: String) -> napi::Result<bool> {
    with_current_compilation(self.compiler_id, |compilation| {
      let path: InternedPath = value.as_str().into();
      let (counter, added) = self.sources(compilation)?;
      Ok(
        !self.deleted_paths.contains(&path)
          && (counter.related_resource_ids(&path).is_some() || added.contains(&path))
          && path.to_string_lossy() == value,
      )
    })
  }

  #[napi]
  pub fn clear(&mut self) -> napi::Result<()> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (counter, added) = self.sources(compilation)?;
      self
        .deleted_paths
        .extend(counter.files().chain(added).cloned());
      Ok(())
    })
  }

  #[napi]
  pub fn update(
    &mut self,
    env: &Env,
    added: Vec<JsString<'_>>,
    deleted: Vec<String>,
  ) -> napi::Result<()> {
    if added.is_empty() {
      // Deletion-only batches still validate the compiler's lifetime.
      with_current_compilation(self.compiler_id, |_| Ok(()))?;
    } else {
      self.add_values(env, added)?;
    }
    self
      .deleted_paths
      .extend(deleted.iter().map(|path| path.as_str().into()));
    Ok(())
  }

  #[napi(ts_return_type = "ReadonlyArray<string>")]
  pub fn values<'env>(&mut self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_current_compilation(self.compiler_id, |compilation| {
      let paths = self.paths(compilation)?;
      let capacity = paths.size_hint().0;
      let mut unique_paths = Vec::with_capacity(capacity);
      let mut indices = InternedPathMap::with_capacity_and_hasher(capacity, Default::default());
      let has_deleted_paths = !self.deleted_paths.is_empty();
      for path in paths {
        if has_deleted_paths && self.deleted_paths.contains(path) {
          continue;
        }
        // A path can occur in both native dependencies and plugin additions.
        indices.entry(path.clone()).or_insert_with(|| {
          let index = unique_paths.len() as u32;
          unique_paths.push(path);
          index
        });
      }
      let dependency_strings = Self::dependency_strings(env, &mut self.dependency_strings)?;
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
