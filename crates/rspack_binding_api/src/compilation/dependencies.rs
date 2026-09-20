use std::{cell::RefCell, rc::Rc};

use napi::{
  Env, JsString,
  bindgen_prelude::{Array, FromNapiValue, JsObjectValue, ToNapiValue},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilerId, FileCounter};
use rspack_napi::OneShotRef;
use rspack_paths::{InternedPath, InternedPathIndexSet, InternedPathMap, InternedPathSet};

use crate::{
  COMPILER_REFERENCES,
  dependency_strings::{
    DependencyValuesCache, PreparedDependencyArrays, dependency_index, intern_js_values,
    referenced_array, refreshed_array, with_compiler,
  },
};

/// Native operations on the owning compiler's current dependency collection.
#[napi]
pub struct FileSystemDependencies {
  kind: FileSystemDependencyKind,
  compiler_id: CompilerId,
  deleted_paths: InternedPathSet,
  values_cache: Option<Rc<RefCell<DependencyValuesCache>>>,
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
      values_cache: None,
      deleted_paths: InternedPathSet::default(),
    }
  }

  fn add_values<'env>(
    &mut self,
    env: &Env,
    values: impl IntoIterator<Item = JsString<'env>>,
  ) -> napi::Result<()> {
    let paths = intern_js_values(env, self.compiler_id, values)?;
    with_current_compilation_mut(self.compiler_id, |compilation| {
      let additions = self.additions(compilation);
      for path in paths {
        self.deleted_paths.remove(&path);
        additions.insert(path);
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
  pub fn added<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_compiler(env, self.compiler_id, |compiler| {
      let (counter, added) = self.sources(&compiler.compiler.compilation)?;
      let mut prepared = PreparedDependencyArrays::default();
      let array = compiler
        .dependency_string_refs
        .borrow_mut()
        .batch(env)
        .prepare_values(&mut prepared, counter.added_files().chain(added))?;
      prepared.materialize(env, compiler)?;
      refreshed_array(env, array)
    })
  }

  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn removed<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_compiler(env, self.compiler_id, |compiler| {
      let (counter, _) = self.sources(&compiler.compiler.compilation)?;
      let mut prepared = PreparedDependencyArrays::default();
      let array = compiler
        .dependency_string_refs
        .borrow_mut()
        .batch(env)
        .prepare_values(&mut prepared, counter.removed_files())?;
      prepared.materialize(env, compiler)?;
      refreshed_array(env, array)
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
    with_compiler(env, self.compiler_id, |compiler| {
      let values_cache = Rc::clone(
        self
          .values_cache
          .get_or_insert_with(|| Rc::new(RefCell::new(DependencyValuesCache::default()))),
      );
      DependencyValuesCache::queue(&values_cache, compiler);
      let result = (|| {
        let (mut array, indices, generation, prepared, direct_updates) = {
          let (counter, added) = self.sources(&compiler.compiler.compilation)?;
          // Both sources are unique. Only the overlap with native paths needs
          // filtering; output order stays native paths followed by JS additions.
          let capacity = counter.files().size_hint().0.saturating_add(added.len());
          let paths = counter.files().chain(
            added
              .iter()
              .filter(|path| counter.related_resource_ids(path).is_none()),
          );
          let mut indices = InternedPathMap::with_capacity_and_hasher(capacity, Default::default());
          let mut cached = values_cache.borrow_mut();
          if cached.array.is_none() {
            cached.array = Some(OneShotRef::new(env.raw(), create_array(env, 0)?)?);
          }
          let array = referenced_array(
            env,
            cached.array.as_ref().expect("initialized values array"),
          )?;
          let empty = cached.indices.is_empty();
          let mut refs = compiler.dependency_string_refs.borrow_mut();
          let generation = refs.generation();
          let registered = cached.registered_generation == Some(generation);
          let mut batch = refs.batch(env);
          let mut prepared = PreparedDependencyArrays::default();
          prepared
            .commands
            .reserve(if empty { capacity.saturating_add(3) } else { 3 });
          prepared
            .commands
            .extend_from_slice(&[u32::from(!empty), 0, 0]);
          let mut direct_updates = Vec::new();
          for path in paths {
            if self.deleted_paths.contains(path) {
              continue;
            }
            let index = dependency_index(indices.len())?;
            indices.insert(path.clone(), index);
            let previous_index = cached.indices.get(path).copied();
            if registered && previous_index == Some(index) {
              continue;
            }
            let create = || match previous_index {
              Some(previous) => array
                .get::<JsString>(previous)?
                .ok_or_else(|| napi::Error::from_reason("Missing cached dependency string")),
              None => env.create_string(path.to_string_lossy().as_ref()),
            };
            if batch.enabled() {
              let pool_index = batch.get_or_insert_index_with(path, create)?;
              if previous_index != Some(index) {
                if !empty {
                  prepared.commands.push(index);
                }
                prepared.commands.push(pool_index);
              }
            } else if previous_index != Some(index) {
              direct_updates.push((index, create()?));
            }
          }
          let length = dependency_index(indices.len())?;
          if batch.enabled() && (prepared.commands.len() != 3 || array.len() != length) {
            prepared.commands[1] = length;
            prepared.commands[2] = dependency_index(prepared.commands.len() - 3)?;
            prepared.targets.push(array);
            prepared.strings = Some(batch.strings()?);
          } else {
            prepared.commands.clear();
          }
          (array, indices, generation, prepared, direct_updates)
        };
        // No graph or RefCell borrows cross the synchronous JS call. Resolve all
        // old strings before writes so moving entries cannot overwrite a source.
        prepared.materialize(env, compiler)?;
        for (index, value) in direct_updates {
          array.set(index, value)?;
        }
        array = refreshed_array(env, array)?;
        let length = dependency_index(indices.len())?;
        if array.len() != length {
          array.set_named_property("length", length)?;
          array = refreshed_array(env, array)?;
        }
        let mut cached = values_cache.borrow_mut();
        cached.indices = indices;
        cached.registered_generation = Some(generation);
        Ok(array)
      })();
      if result.is_err() {
        // Never leave a partially filled array paired with committed indices.
        let _ = values_cache.borrow_mut().invalidate(Some(env));
      }
      result
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
