//! Compiler-owned JS strings. Rust holds weak references to the JS-owned pool.
//! Only accessed on the owning JS thread while the compiler is pinned.
//! The private array and indexed map have identical dense indices. API result
//! arrays never alias this array, and no raw napi_value survives a handle scope.
use napi::{Env, JsString, bindgen_prelude::*};
use rspack_core::{Compilation, CompilerId};
use rspack_napi::WeakRef;
use rspack_paths::{InternedPath, InternedPathIndexSet};

use crate::{
  COMPILER_REFERENCES, JsCompiler,
  js_helpers::{IndexedArrayUpdate, js_owned_ref},
};

pub(crate) fn upgrade_compiler(
  env: &Env,
  compiler_id: CompilerId,
) -> napi::Result<Reference<JsCompiler>> {
  let weak = COMPILER_REFERENCES.with(|references| references.borrow().get(&compiler_id).cloned());
  let owner = weak
    .map(|reference| reference.upgrade(*env))
    .transpose()?
    .flatten()
    .ok_or_else(|| napi::Error::from_reason(format!(
      "Unable to access dependencies for compiler with id = {compiler_id:?}. The Compiler has been garbage collected by JavaScript."
    )))?;
  // Pin for the active binding call: N-API allocations may otherwise collect the owner
  // while a dependency wrapper, which stores only its id, is still reachable.
  Ok(owner)
}

pub(crate) fn referenced_array<'env>(
  env: &'env Env,
  reference: &WeakRef,
) -> napi::Result<Array<'env>> {
  // SAFETY: the reference points to an array in this environment. For weak
  // references, the caller pins its JS owner for this active handle scope.
  unsafe { Array::from_napi_value(env.raw(), ToNapiValue::to_napi_value(env.raw(), reference)?) }
}

pub(crate) struct FileSystemDependencyStringPool {
  strings: WeakRef,
  entries: InternedPathIndexSet,
}

impl FileSystemDependencyStringPool {
  pub(crate) fn new(env: &Env, compiler: &mut Object<'_>) -> napi::Result<Self> {
    let strings = env.create_array(0)?.coerce_to_object()?;
    Ok(Self {
      strings: js_owned_ref(
        env,
        compiler,
        "rspack.fileSystemDependencyStringPool",
        strings,
      )?,
      entries: Default::default(),
    })
  }

  pub(crate) fn clear(&mut self, env: &Env) -> napi::Result<()> {
    self.entries = Default::default();
    referenced_array(env, &self.strings)?.set_named_property("length", 0u32)?;
    Ok(())
  }

  fn removed_indices(&self, compilation: &Compilation) -> napi::Result<Vec<u32>> {
    if compilation.build_module_graph_artifact.is_stolen() {
      return Err(napi::Error::from_reason(
        "Cannot prune dependency strings while a compilation pass holds the module graph artifact",
      ));
    }
    let artifact = &compilation.build_module_graph_artifact;
    let dependency_sources = [
      (&artifact.file_dependencies, &compilation.file_dependencies),
      (
        &artifact.context_dependencies,
        &compilation.context_dependencies,
      ),
      (
        &artifact.missing_dependencies,
        &compilation.missing_dependencies,
      ),
      (
        &artifact.build_dependencies,
        &compilation.build_dependencies,
      ),
    ];
    // Descending indices let both sides swap-remove without shifting the
    // remaining deletion indices. Membership is in the graph, not on disk.
    Ok(
      self
        .entries
        .iter()
        .enumerate()
        .rev()
        .filter(|&(_, path)| {
          !dependency_sources
            .iter()
            .any(|(dependency_counter, dependencies)| {
              dependency_counter.related_resource_ids(path).is_some() || dependencies.contains(path)
            })
        })
        .map(|(index, _)| index as u32)
        .collect(),
    )
  }

  fn remove_indices(&mut self, indices: &[u32]) {
    for &index in indices {
      self.entries.swap_remove_index(index as usize);
    }
    if self.entries.capacity() >= 4096
      && self.entries.capacity() > self.entries.len().saturating_mul(4)
    {
      self.entries.shrink_to_fit();
    }
  }

  pub(crate) fn session<'pool, 'env>(
    &'pool mut self,
    env: &'env Env,
  ) -> FileSystemDependencyStringPoolSession<'pool, 'env> {
    FileSystemDependencyStringPoolSession {
      pool: self,
      env,
      strings: None,
    }
  }
}

pub(crate) struct FileSystemDependencyStringPoolSession<'pool, 'env> {
  pool: &'pool mut FileSystemDependencyStringPool,
  env: &'env Env,
  strings: Option<Array<'env>>,
}

impl<'env> FileSystemDependencyStringPoolSession<'_, 'env> {
  fn array(&mut self) -> napi::Result<&mut Array<'env>> {
    if self.strings.is_none() {
      self.strings = Some(referenced_array(self.env, &self.pool.strings)?);
    }
    Ok(self.strings.as_mut().expect("initialized array handle"))
  }

  pub(crate) fn intern_js(&mut self, value: JsString<'env>) -> napi::Result<InternedPath> {
    let path: InternedPath = value.into_utf8()?.as_str()?.into();
    if !self.pool.entries.contains(&path) {
      self.insert(path.clone(), value)?;
    }
    Ok(path)
  }

  fn intern_js_values(&mut self, values: Vec<JsString<'env>>) -> napi::Result<Vec<InternedPath>> {
    let mut paths = Vec::with_capacity(values.len());
    for value in values {
      paths.push(self.intern_js(value)?);
    }
    Ok(paths)
  }

  fn insert(&mut self, path: InternedPath, value: JsString<'env>) -> napi::Result<u32> {
    let index = self.pool.entries.len() as u32;
    self.array()?.set(index, value)?;
    self.pool.entries.insert(path);
    Ok(index)
  }

  pub(crate) fn strings(&mut self) -> napi::Result<Array<'env>> {
    Ok(*self.array()?)
  }

  pub(crate) fn get_or_insert_index(&mut self, path: &InternedPath) -> napi::Result<u32> {
    if let Some(index) = self.pool.entries.get_index_of(path) {
      return Ok(index as u32);
    }
    let value = self.env.create_string(path.to_string_lossy().as_ref())?;
    self.insert(path.clone(), value)
  }

  pub(crate) fn queue_dependency_array_update<'path>(
    &mut self,
    update: &mut IndexedArrayUpdate<'env>,
    paths: &mut dyn Iterator<Item = &'path InternedPath>,
  ) -> napi::Result<Array<'env>> {
    let array = self.env.create_array(paths.size_hint().0 as u32)?;
    update
      .commands
      .reserve(paths.size_hint().0.saturating_add(2));
    update.commands.extend_from_slice(&[0, 0]);
    for path in paths {
      let index = self.get_or_insert_index(path)?;
      update.commands.push(index);
    }
    let length = (update.commands.len() - 2) as u32;
    if length == 0 {
      update.commands.clear();
    } else {
      update.commands[1] = length;
      update.target = Some(array);
      update.source = Some(self.strings()?);
    }
    Ok(array)
  }
}

/// JS helpers can change Array.length; napi-rs Array caches it in the wrapper.
pub(crate) fn refreshed_array<'env>(
  env: &'env Env,
  array: Array<'env>,
) -> napi::Result<Array<'env>> {
  // SAFETY: this is the same array in the same active N-API handle scope.
  unsafe { Array::from_napi_value(env.raw(), ToNapiValue::to_napi_value(env.raw(), array)?) }
}

#[derive(Default)]
pub(crate) struct FileSystemDependencyArrayCache {
  array: Option<WeakRef>,
  // Paths in the same order as the cached JS array.
  pub(crate) paths: Vec<InternedPath>,
}

impl FileSystemDependencyArrayCache {
  pub(crate) fn array<'env>(
    &mut self,
    env: &'env Env,
    owner: &mut Object<'_>,
  ) -> napi::Result<Array<'env>> {
    if self.array.is_none() {
      let array = env.create_array(0)?.coerce_to_object()?;
      self.array = Some(js_owned_ref(
        env,
        owner,
        "rspack.fileSystemDependencyArray",
        array,
      )?);
    }
    // The receiver owns the array and stays alive for this binding call.
    referenced_array(
      env,
      self.array.as_ref().expect("initialized dependency array"),
    )
  }
}

impl JsCompiler {
  pub(crate) fn prune_file_system_dependency_strings(&self, env: &Env) -> napi::Result<()> {
    let (strings, removed_indices) = {
      let pool = self.file_system_dependency_string_pool.borrow();
      let removed_indices = pool.removed_indices(&self.compiler.compilation)?;
      if removed_indices.is_empty() {
        return Ok(());
      }
      (referenced_array(env, &pool.strings)?, removed_indices)
    };
    // The completion guard pins the JS compiler, and no native borrows cross the call.
    let indices = Uint32ArraySlice::from_data(env, removed_indices)?;
    self
      .js_helpers
      .swap_remove_array_elements(env, strings, &indices)?;
    self
      .file_system_dependency_string_pool
      .borrow_mut()
      .remove_indices(indices.as_ref());
    Ok(())
  }

  pub(crate) fn clear_file_system_dependency_strings(&self, env: &Env) -> napi::Result<()> {
    self
      .file_system_dependency_string_pool
      .borrow_mut()
      .clear(env)
  }
}

pub(crate) fn intern_js_values<'env>(
  env: &Env,
  compiler_id: CompilerId,
  values: Vec<JsString<'env>>,
) -> napi::Result<Vec<InternedPath>> {
  let compiler = upgrade_compiler(env, compiler_id)?;
  let mut pool = compiler.file_system_dependency_string_pool.borrow_mut();
  pool.session(env).intern_js_values(values)
}
