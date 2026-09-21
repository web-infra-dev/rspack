use napi::{
  Env, JsString,
  bindgen_prelude::{Array, JsObjectValue, This},
};
use napi_derive::napi;
use rspack_core::{Compilation, CompilerId, FileCounter};
use rspack_paths::{InternedPath, InternedPathIndexSet, InternedPathSet};

use crate::{
  COMPILER_REFERENCES,
  file_system_dependency_strings::{
    FileSystemDependencyArrayCache, intern_js_values, refreshed_array, upgrade_compiler,
  },
  js_helpers::IndexedArrayUpdateBatch,
};

/// Native operations on the owning compiler's current dependency collection.
#[napi]
pub struct FileSystemDependencies {
  get_dependencies: fn(&Compilation) -> (&FileCounter, &InternedPathIndexSet),
  get_dependencies_mut: fn(&mut Compilation) -> &mut InternedPathIndexSet,
  compiler_id: CompilerId,
  excluded_paths: InternedPathSet,
  array_cache: FileSystemDependencyArrayCache,
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

impl FileSystemDependencies {
  pub fn new(
    compiler_id: CompilerId,
    get_dependencies: fn(&Compilation) -> (&FileCounter, &InternedPathIndexSet),
    get_dependencies_mut: fn(&mut Compilation) -> &mut InternedPathIndexSet,
  ) -> Self {
    Self {
      get_dependencies,
      get_dependencies_mut,
      compiler_id,
      array_cache: Default::default(),
      excluded_paths: InternedPathSet::default(),
    }
  }

  fn add_values<'env>(&mut self, env: &Env, values: Vec<JsString<'env>>) -> napi::Result<()> {
    let paths = intern_js_values(env, self.compiler_id, values)?;
    with_current_compilation_mut(self.compiler_id, |compilation| {
      let dependencies = (self.get_dependencies_mut)(compilation);
      for path in paths {
        self.excluded_paths.remove(&path);
        dependencies.insert(path);
      }
      Ok(())
    })
  }

  fn dependency_sources<'a>(
    &self,
    compilation: &'a Compilation,
  ) -> napi::Result<(&'a FileCounter, &'a InternedPathIndexSet)> {
    if compilation.build_module_graph_artifact.is_stolen() {
      return Err(napi::Error::from_reason(
        "Compilation dependencies are not available while a compilation pass is holding the module graph artifact",
      ));
    }
    Ok((self.get_dependencies)(compilation))
  }
}

#[napi]
impl FileSystemDependencies {
  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn added<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    let compiler = upgrade_compiler(env, self.compiler_id)?;
    let (dependency_counter, dependencies) =
      self.dependency_sources(&compiler.compiler.compilation)?;
    let mut paths = dependency_counter.added_files().chain(dependencies);
    let mut updates = IndexedArrayUpdateBatch::default();
    let array = compiler
      .file_system_dependency_string_pool
      .borrow_mut()
      .session(env)
      .queue_dependency_array_update(&mut updates, &mut paths)?;
    updates.apply(env, &compiler.js_helpers)?;
    refreshed_array(env, array)
  }

  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn removed<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    let compiler = upgrade_compiler(env, self.compiler_id)?;
    let (dependency_counter, _) = self.dependency_sources(&compiler.compiler.compilation)?;
    let mut paths = dependency_counter.removed_files();
    let mut updates = IndexedArrayUpdateBatch::default();
    let array = compiler
      .file_system_dependency_string_pool
      .borrow_mut()
      .session(env)
      .queue_dependency_array_update(&mut updates, &mut paths)?;
    updates.apply(env, &compiler.js_helpers)?;
    refreshed_array(env, array)
  }

  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (dependency_counter, dependencies) = self.dependency_sources(compilation)?;
      let tracked_count = if self.excluded_paths.is_empty() {
        dependency_counter.files().count()
      } else {
        dependency_counter
          .files()
          .filter(|path| !self.excluded_paths.contains(*path))
          .count()
      };
      // Both sources are already unique. Count only compilation dependencies
      // absent from the module graph counter, without allocating a union set.
      let additional_count = dependencies
        .iter()
        .filter(|path| {
          !self.excluded_paths.contains(*path)
            && dependency_counter.related_resource_ids(path).is_none()
        })
        .count();
      Ok((tracked_count + additional_count) as u32)
    })
  }

  #[napi]
  pub fn has(&self, value: String) -> napi::Result<bool> {
    with_current_compilation(self.compiler_id, |compilation| {
      // The JavaScript filesystem dependency API represents paths as UTF-8
      // strings. Rspack does not support non-UTF-8 paths at this boundary.
      let path: InternedPath = value.as_str().into();
      let (dependency_counter, dependencies) = self.dependency_sources(compilation)?;
      Ok(
        !self.excluded_paths.contains(&path)
          && (dependency_counter.related_resource_ids(&path).is_some()
            || dependencies.contains(&path))
          && path.to_string_lossy() == value,
      )
    })
  }

  #[napi]
  pub fn clear(&mut self) -> napi::Result<()> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (dependency_counter, dependencies) = self.dependency_sources(compilation)?;
      self
        .excluded_paths
        .extend(dependency_counter.files().chain(dependencies).cloned());
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
      .excluded_paths
      .extend(deleted.iter().map(|path| path.as_str().into()));
    Ok(())
  }

  #[napi(ts_return_type = "ReadonlyArray<string>")]
  pub fn values<'env>(&mut self, env: &'env Env, mut this: This) -> napi::Result<Array<'env>> {
    let compiler = upgrade_compiler(env, self.compiler_id)?;
    let result = (|| {
      let (mut array, length, updates) = {
        let (dependency_counter, dependencies) =
          self.dependency_sources(&compiler.compiler.compilation)?;
        // Both sources are unique. Filter compilation dependencies already in
        // the module graph counter, preserving counter-first iteration order.
        let capacity = dependency_counter
          .files()
          .size_hint()
          .0
          .saturating_add(dependencies.len());
        let paths = dependency_counter.files().chain(
          dependencies
            .iter()
            .filter(|path| dependency_counter.related_resource_ids(path).is_none()),
        );
        let cached = &mut self.array_cache;
        let array = cached.array(env, &mut this)?;
        let empty = cached.paths.is_empty();
        let mut pool = compiler.file_system_dependency_string_pool.borrow_mut();
        let mut session = pool.session(env);
        let mut updates = IndexedArrayUpdateBatch::default();
        updates
          .commands
          .reserve(if empty { capacity.saturating_add(3) } else { 3 });
        updates
          .commands
          .extend_from_slice(&[u32::from(!empty), 0, 0]);
        let mut length = 0;
        for path in paths {
          if self.excluded_paths.contains(path) {
            continue;
          }
          let index = length;
          length += 1;
          if cached.paths.get(index) == Some(path) {
            continue;
          }
          let pool_index = session.get_or_insert_index(path)?;
          // Reuse the vector and clone only changed paths. Any failure below
          // clears this provisional state before it can be reused.
          if let Some(previous) = cached.paths.get_mut(index) {
            *previous = path.clone();
          } else {
            cached.paths.push(path.clone());
          }
          if !empty {
            updates.commands.push(index as u32);
          }
          updates.commands.push(pool_index);
        }
        cached.paths.truncate(length);
        let length = length as u32;
        if updates.commands.len() != 3 || array.len() != length {
          updates.commands[1] = length;
          updates.commands[2] = (updates.commands.len() - 3) as u32;
          updates.targets.push(array);
          updates.source = Some(session.strings()?);
        } else {
          updates.commands.clear();
        }
        (array, length, updates)
      };
      // No graph or RefCell borrows cross the synchronous JS call. Copy strings
      // directly from the shared pool, including entries that change position.
      updates.apply(env, &compiler.js_helpers)?;
      array = refreshed_array(env, array)?;
      if array.len() != length {
        array.set_named_property("length", length)?;
        array = refreshed_array(env, array)?;
      }
      Ok(array)
    })();
    if result.is_err() {
      // A failed update must not leave paths describing an incomplete JS array.
      self.array_cache.paths.clear();
    }
    result
  }

  #[napi]
  pub fn add(&mut self, env: &Env, value: JsString<'_>) -> napi::Result<()> {
    self.add_values(env, vec![value])
  }

  #[napi]
  pub fn add_all(&mut self, env: &Env, values: Vec<JsString<'_>>) -> napi::Result<()> {
    self.add_values(env, values)
  }
}
