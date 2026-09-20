//! Compiler-local JS string roots. Only accessed on the owning JS thread.
//! The private array and indexed map have identical dense indices. API result
//! arrays never alias this array, and no raw napi_value survives a handle scope.
use std::{cell::RefCell, rc::Rc};

use napi::{Env, JsString, bindgen_prelude::*};
use rspack_core::{CompilerId, LoaderDependencies};
use rspack_napi::OneShotRef;
use rspack_paths::{InternedPath, InternedPathIndexMap, InternedPathMap};

use crate::{COMPILER_REFERENCES, JsCompiler};

pub(crate) fn with_compiler<R>(
  env: &Env,
  compiler_id: CompilerId,
  f: impl FnOnce(&JsCompiler) -> napi::Result<R>,
) -> napi::Result<R> {
  let weak = COMPILER_REFERENCES.with(|refs| refs.borrow().get(&compiler_id).cloned());
  let owner = weak
    .map(|reference| reference.upgrade(*env))
    .transpose()?
    .flatten()
    .ok_or_else(|| napi::Error::from_reason(format!(
      "Unable to access dependencies for compiler with id = {compiler_id:?}. The Compiler has been garbage collected by JavaScript."
    )))?;
  // Pin only for this call: N-API allocations may otherwise collect the owner
  // while a dependency wrapper, which stores only its id, is still reachable.
  f(&owner)
}

pub(crate) fn referenced_array<'env>(
  env: &'env Env,
  reference: &OneShotRef,
) -> napi::Result<Array<'env>> {
  // SAFETY: the reference roots an array in this environment. The returned
  // handle is used only inside the caller's active N-API handle scope.
  unsafe { Array::from_napi_value(env.raw(), ToNapiValue::to_napi_value(env.raw(), reference)?) }
}

#[derive(Default)]
pub(crate) struct InternedPathJsStringRefs {
  generation: u64,
  enabled: bool,
  strings: Option<OneShotRef>,
  entries: InternedPathIndexMap<u64>,
}

impl InternedPathJsStringRefs {
  pub(crate) fn generation(&self) -> u64 {
    self.generation
  }

  pub(crate) fn clear(&mut self) {
    self.strings = None;
    self.entries = Default::default();
  }

  pub(crate) fn disable(&mut self) {
    self.clear();
    self.enabled = false;
  }

  pub(crate) fn begin_generation(&mut self, env: &Env) -> napi::Result<()> {
    self.enabled = true;
    self.generation += 1;
    let result = self.sweep(env);
    if result.is_err() {
      self.clear();
    }
    result
  }

  fn sweep(&mut self, env: &Env) -> napi::Result<()> {
    let oldest = self.generation.saturating_sub(1);
    let Some(first) = self
      .entries
      .values()
      .position(|&generation| generation < oldest)
    else {
      return Ok(());
    };
    if self.entries.values().all(|&generation| generation < oldest) {
      self.clear();
      return Ok(());
    }
    let mut array = referenced_array(env, self.strings.as_ref().expect("nonempty string roots"))?;
    let mut index = first;
    while index < self.entries.len() {
      if *self.entries.get_index(index).expect("valid index").1 >= oldest {
        index += 1;
        continue;
      }
      while self
        .entries
        .last()
        .is_some_and(|(_, &generation)| generation < oldest)
      {
        self.entries.pop();
      }
      if index >= self.entries.len() {
        break;
      }
      let last = self.entries.len() - 1;
      let value = array
        .get::<JsString>(last as u32)?
        .ok_or_else(|| napi::Error::from_reason("Missing dependency string root"))?;
      array.set(index as u32, value)?;
      self.entries.swap_remove_index(index);
      index += 1;
    }
    array.set_named_property("length", self.entries.len() as u32)?;
    if self.entries.capacity() >= 4096
      && self.entries.capacity() > self.entries.len().saturating_mul(4)
    {
      self.entries.shrink_to_fit();
    }
    Ok(())
  }

  pub(crate) fn batch<'cache, 'env>(
    &'cache mut self,
    env: &'env Env,
  ) -> DependencyStringBatch<'cache, 'env> {
    DependencyStringBatch {
      refs: self,
      env,
      strings: None,
    }
  }
}

pub(crate) struct DependencyStringBatch<'cache, 'env> {
  refs: &'cache mut InternedPathJsStringRefs,
  env: &'env Env,
  strings: Option<Array<'env>>,
}

impl<'env> DependencyStringBatch<'_, 'env> {
  fn array(&mut self) -> napi::Result<&mut Array<'env>> {
    if self.strings.is_none() {
      if self.refs.strings.is_none() {
        let array = self.env.create_array(0)?;
        self.refs.strings = Some(OneShotRef::new(self.env.raw(), array)?);
      }
      self.strings = Some(referenced_array(
        self.env,
        self.refs.strings.as_ref().expect("initialized roots"),
      )?);
    }
    Ok(self.strings.as_mut().expect("initialized array handle"))
  }

  pub(crate) fn intern_js(&mut self, value: JsString<'env>) -> napi::Result<InternedPath> {
    let path: InternedPath = value.into_utf8()?.as_str()?.into();
    if self.refs.enabled {
      if let Some((_, _, generation)) = self.refs.entries.get_full_mut(&path) {
        *generation = self.refs.generation;
      } else {
        self.insert(path.clone(), value)?;
      }
    }
    Ok(path)
  }

  fn insert(&mut self, path: InternedPath, value: JsString<'env>) -> napi::Result<u32> {
    let length = dependency_index(self.refs.entries.len().saturating_add(1))?;
    let index = length - 1;
    self.array()?.set(index, value)?;
    self.refs.entries.insert(path, self.refs.generation);
    Ok(index)
  }

  pub(crate) fn enabled(&self) -> bool {
    self.refs.enabled
  }

  pub(crate) fn strings(&mut self) -> napi::Result<Array<'env>> {
    Ok(*self.array()?)
  }

  pub(crate) fn get_or_insert_index_with(
    &mut self,
    path: &InternedPath,
    create: impl FnOnce() -> napi::Result<JsString<'env>>,
  ) -> napi::Result<u32> {
    debug_assert!(self.refs.enabled);
    if let Some((index, _, generation)) = self.refs.entries.get_full_mut(path) {
      *generation = self.refs.generation;
      return dependency_index(index);
    }
    self.insert(path.clone(), create()?)
  }

  pub(crate) fn prepare_values<'path>(
    &mut self,
    prepared: &mut PreparedDependencyArrays<'env>,
    paths: impl IntoIterator<Item = &'path InternedPath>,
  ) -> napi::Result<Array<'env>> {
    let paths = paths.into_iter();
    let mut array = self
      .env
      .create_array(dependency_index(paths.size_hint().0)?)?;
    if !self.enabled() {
      // close releases the helper and disables pooling, but retained wrappers
      // may still read the current compilation while its compiler is alive.
      for (index, path) in paths.enumerate() {
        array.set(
          dependency_index(index)?,
          self.env.create_string(path.to_string_lossy().as_ref())?,
        )?;
      }
      return Ok(array);
    }
    let start = prepared.commands.len();
    prepared
      .commands
      .reserve(paths.size_hint().0.saturating_add(3));
    prepared.commands.extend_from_slice(&[0, 0, 0]);
    let env = self.env;
    for path in paths {
      let index = self
        .get_or_insert_index_with(path, || env.create_string(path.to_string_lossy().as_ref()))?;
      prepared.commands.push(index);
    }
    let length = dependency_index(prepared.commands.len() - start - 3)?;
    if length == 0 {
      prepared.commands.truncate(start);
    } else {
      prepared.commands[start + 1] = length;
      prepared.commands[start + 2] = length;
      prepared.targets.push(array);
      prepared.strings = Some(self.strings()?);
    }
    Ok(array)
  }
}

pub(crate) fn dependency_index(value: usize) -> napi::Result<u32> {
  u32::try_from(value)
    .map_err(|_| napi::Error::from_reason("Dependency array exceeds the Uint32 index range"))
}

/// A synchronous transfer only: no cache borrow, native graph reference or
/// shared-pool index survives the helper call. Empty/disabled batches have no pool.
#[derive(Default)]
pub(crate) struct PreparedDependencyArrays<'env> {
  pub(crate) strings: Option<Array<'env>>,
  pub(crate) targets: Vec<Array<'env>>,
  // One [mode, final length, payload length, ...payload] segment per target.
  // Fill (0) uses pool indices; Patch (1) uses destination/pool index pairs.
  pub(crate) commands: Vec<u32>,
}

impl PreparedDependencyArrays<'_> {
  pub(crate) fn materialize(self, env: &Env, compiler: &JsCompiler) -> napi::Result<()> {
    if self.commands.is_empty() {
      return Ok(());
    }
    let function = {
      let reference = compiler.dependency_array_materializer.borrow();
      let reference = reference.as_ref().ok_or_else(|| {
        napi::Error::from_reason("Dependency array materializer has been released")
      })?;
      // SAFETY: construction validates this private, synchronous JS function.
      unsafe {
        Function::<FnArgs<(Array, Vec<Array>, Uint32ArraySlice)>, ()>::from_napi_value(
          env.raw(),
          ToNapiValue::to_napi_value(env.raw(), reference)?,
        )?
      }
    };
    let commands = Uint32ArraySlice::from_data(env, self.commands)?;
    function.call(
      (
        self
          .strings
          .expect("nonempty dependency transfer has a string pool"),
        self.targets,
        commands,
      )
        .into(),
    )
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
pub(crate) struct DependencyValuesCache {
  pub(crate) array: Option<OneShotRef>,
  pub(crate) indices: InternedPathMap<u32>,
  pub(crate) registered_generation: Option<u64>,
  queued_for_cleanup: bool,
}

impl DependencyValuesCache {
  pub(crate) fn queue(this: &Rc<RefCell<Self>>, compiler: &JsCompiler) {
    let mut values = this.borrow_mut();
    if !values.queued_for_cleanup {
      compiler
        .dependency_values_to_clear
        .borrow_mut()
        .push(Rc::downgrade(this));
      values.queued_for_cleanup = true;
    }
  }

  pub(crate) fn invalidate(&mut self, env: Option<&Env>) -> napi::Result<()> {
    self.indices = Default::default();
    self.registered_generation = None;
    self.queued_for_cleanup = false;
    if let (Some(env), Some(reference)) = (env, self.array.as_ref()) {
      let result = referenced_array(env, reference)
        .and_then(|mut array| array.set_named_property("length", 0u32));
      if result.is_err() {
        self.array = None;
      }
      result
    } else {
      // Finalization must only drop roots, without mutating JS objects.
      self.array = None;
      Ok(())
    }
  }
}

impl JsCompiler {
  pub(crate) fn clear_dependency_values(&self, env: Option<&Env>) -> napi::Result<()> {
    let pending = std::mem::take(&mut *self.dependency_values_to_clear.borrow_mut());
    let mut error = None;
    for weak in pending {
      if let Some(values) = weak.upgrade()
        && let Err(err) = values.borrow_mut().invalidate(env)
      {
        error.get_or_insert(err);
      }
    }
    error.map_or(Ok(()), Err)
  }

  pub(crate) fn clear_dependency_strings(
    &self,
    env: Option<&Env>,
    disable: bool,
  ) -> napi::Result<()> {
    let result = self.clear_dependency_values(env);
    let mut refs = self.dependency_string_refs.borrow_mut();
    if disable {
      self.dependency_array_materializer.borrow_mut().take();
      refs.disable();
    } else {
      refs.clear();
    }
    result
  }
}

#[derive(Default)]
pub(crate) struct DependencyPaths {
  pub(crate) file: Vec<InternedPath>,
  pub(crate) context: Vec<InternedPath>,
  pub(crate) missing: Vec<InternedPath>,
  pub(crate) build: Vec<InternedPath>,
}

impl DependencyPaths {
  pub(crate) fn from_js(
    env: &Env,
    compiler_id: CompilerId,
    object: Object<'_>,
  ) -> napi::Result<Self> {
    // Read user properties/elements before borrowing the cache: getters may
    // re-enter binding APIs. JsString handles remain in this handle scope.
    let file = object.get_named_property::<Vec<JsString>>("fileDependencies")?;
    let context = object.get_named_property::<Vec<JsString>>("contextDependencies")?;
    let missing = object.get_named_property::<Vec<JsString>>("missingDependencies")?;
    let build = object.get_named_property::<Vec<JsString>>("buildDependencies")?;
    with_compiler(env, compiler_id, |compiler| {
      let mut refs = compiler.dependency_string_refs.borrow_mut();
      let mut batch = refs.batch(env);
      Ok(Self {
        file: file
          .into_iter()
          .map(|value| batch.intern_js(value))
          .collect::<napi::Result<_>>()?,
        context: context
          .into_iter()
          .map(|value| batch.intern_js(value))
          .collect::<napi::Result<_>>()?,
        missing: missing
          .into_iter()
          .map(|value| batch.intern_js(value))
          .collect::<napi::Result<_>>()?,
        build: build
          .into_iter()
          .map(|value| batch.intern_js(value))
          .collect::<napi::Result<_>>()?,
      })
    })
  }

  pub(crate) fn to_js<'env>(
    &self,
    env: &'env Env,
    compiler_id: CompilerId,
  ) -> napi::Result<Object<'env>> {
    with_compiler(env, compiler_id, |compiler| {
      let mut prepared = PreparedDependencyArrays::default();
      let (file, context, missing, build) = {
        let mut refs = compiler.dependency_string_refs.borrow_mut();
        let mut batch = refs.batch(env);
        (
          batch.prepare_values(&mut prepared, &self.file)?,
          batch.prepare_values(&mut prepared, &self.context)?,
          batch.prepare_values(&mut prepared, &self.missing)?,
          batch.prepare_values(&mut prepared, &self.build)?,
        )
      };
      prepared.materialize(env, compiler)?;
      let mut result = Object::new(env)?;
      result.set_named_property("fileDependencies", file)?;
      result.set_named_property("contextDependencies", context)?;
      result.set_named_property("missingDependencies", missing)?;
      result.set_named_property("buildDependencies", build)?;
      Ok(result)
    })
  }

  pub(crate) fn is_empty(&self) -> bool {
    self.file.is_empty()
      && self.context.is_empty()
      && self.missing.is_empty()
      && self.build.is_empty()
  }
}

impl From<&LoaderDependencies> for DependencyPaths {
  fn from(value: &LoaderDependencies) -> Self {
    Self {
      file: value.file.iter().cloned().collect(),
      context: value.context.iter().cloned().collect(),
      missing: value.missing.iter().cloned().collect(),
      build: value.build.iter().cloned().collect(),
    }
  }
}

impl From<DependencyPaths> for LoaderDependencies {
  fn from(value: DependencyPaths) -> Self {
    Self {
      file: value.file.into_iter().collect(),
      context: value.context.into_iter().collect(),
      missing: value.missing.into_iter().collect(),
      build: value.build.into_iter().collect(),
    }
  }
}

pub struct CompilerDependencyPaths {
  pub(crate) compiler_id: CompilerId,
  pub(crate) paths: DependencyPaths,
}

impl ToNapiValue for CompilerDependencyPaths {
  unsafe fn to_napi_value(
    env: napi::sys::napi_env,
    value: Self,
  ) -> napi::Result<napi::sys::napi_value> {
    let env = unsafe { Env::from_raw(env) };
    unsafe { ToNapiValue::to_napi_value(env.raw(), value.paths.to_js(&env, value.compiler_id)?) }
  }
}

pub(crate) fn intern_js_values<'env>(
  env: &Env,
  compiler_id: CompilerId,
  values: impl IntoIterator<Item = JsString<'env>>,
) -> napi::Result<Vec<InternedPath>> {
  with_compiler(env, compiler_id, |compiler| {
    let mut refs = compiler.dependency_string_refs.borrow_mut();
    let mut batch = refs.batch(env);
    values
      .into_iter()
      .map(|value| batch.intern_js(value))
      .collect()
  })
}
