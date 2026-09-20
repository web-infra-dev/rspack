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
    FileSystemDependencyArrayCache, intern_js_values, refreshed_array, with_compiler,
  },
  js_helpers::IndexedArrayUpdateBatch,
};

/// Native operations on the owning compiler's current dependency collection.
#[napi]
pub struct FileSystemDependencies {
  select_sources: fn(&Compilation) -> (&FileCounter, &InternedPathIndexSet),
  select_additions: fn(&mut Compilation) -> &mut InternedPathIndexSet,
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
    select_sources: fn(&Compilation) -> (&FileCounter, &InternedPathIndexSet),
    select_additions: fn(&mut Compilation) -> &mut InternedPathIndexSet,
  ) -> Self {
    Self {
      select_sources,
      select_additions,
      compiler_id,
      array_cache: Default::default(),
      excluded_paths: InternedPathSet::default(),
    }
  }

  fn add_values<'env>(
    &mut self,
    env: &Env,
    values: impl IntoIterator<Item = JsString<'env>>,
  ) -> napi::Result<()> {
    let paths = intern_js_values(env, self.compiler_id, values)?;
    with_current_compilation_mut(self.compiler_id, |compilation| {
      let additions = (self.select_additions)(compilation);
      for path in paths {
        self.excluded_paths.remove(&path);
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
    Ok((self.select_sources)(compilation))
  }
}

#[napi]
impl FileSystemDependencies {
  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn added<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_compiler(env, self.compiler_id, |compiler| {
      let (counter, plugin_paths) = self.sources(&compiler.compiler.compilation)?;
      let mut updates = IndexedArrayUpdateBatch::default();
      let array = compiler
        .file_system_dependency_string_pool
        .borrow_mut()
        .session(env)
        .prepare_values(&mut updates, counter.added_files().chain(plugin_paths))?;
      updates.apply(env, &compiler.js_helpers)?;
      refreshed_array(env, array)
    })
  }

  #[napi(getter, ts_return_type = "Array<string>")]
  pub fn removed<'env>(&self, env: &'env Env) -> napi::Result<Array<'env>> {
    with_compiler(env, self.compiler_id, |compiler| {
      let (counter, _) = self.sources(&compiler.compiler.compilation)?;
      let mut updates = IndexedArrayUpdateBatch::default();
      let array = compiler
        .file_system_dependency_string_pool
        .borrow_mut()
        .session(env)
        .prepare_values(&mut updates, counter.removed_files())?;
      updates.apply(env, &compiler.js_helpers)?;
      refreshed_array(env, array)
    })
  }

  #[napi]
  pub fn size(&self) -> napi::Result<u32> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (counter, plugin_paths) = self.sources(compilation)?;
      let native_count = if self.excluded_paths.is_empty() {
        counter.files().count()
      } else {
        counter
          .files()
          .filter(|path| !self.excluded_paths.contains(*path))
          .count()
      };
      // Both sources are already unique. Count only plugin paths absent from
      // the native counter, without allocating a temporary union set.
      let plugin_count = plugin_paths
        .iter()
        .filter(|path| {
          !self.excluded_paths.contains(*path) && counter.related_resource_ids(path).is_none()
        })
        .count();
      Ok((native_count + plugin_count) as u32)
    })
  }

  #[napi]
  pub fn has(&self, value: String) -> napi::Result<bool> {
    with_current_compilation(self.compiler_id, |compilation| {
      let path: InternedPath = value.as_str().into();
      let (counter, plugin_paths) = self.sources(compilation)?;
      Ok(
        !self.excluded_paths.contains(&path)
          && (counter.related_resource_ids(&path).is_some() || plugin_paths.contains(&path))
          && path.to_string_lossy() == value,
      )
    })
  }

  #[napi]
  pub fn clear(&mut self) -> napi::Result<()> {
    with_current_compilation(self.compiler_id, |compilation| {
      let (counter, plugin_paths) = self.sources(compilation)?;
      self
        .excluded_paths
        .extend(counter.files().chain(plugin_paths).cloned());
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
    with_compiler(env, self.compiler_id, |compiler| {
      let result = (|| {
        let (mut array, length, updates) = {
          let (counter, plugin_paths) = self.sources(&compiler.compiler.compilation)?;
          // Both sources are unique. Only the overlap with native paths needs
          // filtering; output order stays native paths followed by JS additions.
          let capacity = counter
            .files()
            .size_hint()
            .0
            .saturating_add(plugin_paths.len());
          let paths = counter.files().chain(
            plugin_paths
              .iter()
              .filter(|path| counter.related_resource_ids(path).is_none()),
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
