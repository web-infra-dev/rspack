use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxChunkInitFragment, ChunkUkey, Compilation, ImportSpec, ModuleGraph, ModuleIdentifier,
  RuntimeCodeTemplate, RuntimeGlobals, find_new_name,
  rspack_sources::{ConcatSource, RawStringSource},
};
use rspack_util::fx_hash::{FxHashMap, FxHashSet, FxIndexMap, FxIndexSet};
use swc_core::atoms::Atom;

#[derive(Debug, Clone)]
pub enum Ref {
  Symbol(SymbolRef),
  Inline(String),
}

impl Ref {
  pub fn render(&self) -> Cow<'_, str> {
    match self {
      Ref::Symbol(symbol_ref) => Cow::Owned(symbol_ref.render()),
      Ref::Inline(inline) => Cow::Borrowed(inline),
    }
  }
}

#[derive(Clone)]
pub struct SymbolRef {
  pub module: ModuleIdentifier,
  pub symbol: Atom,
  pub ids: Vec<Atom>,
  renderer: Arc<dyn Fn(&SymbolRef) -> String + Send + Sync>,
}

impl std::fmt::Debug for SymbolRef {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("SymbolRef")
      .field("module", &self.module)
      .field("symbol", &self.symbol)
      .field("ids", &self.ids)
      .finish()
  }
}

impl SymbolRef {
  pub fn new(
    module: ModuleIdentifier,
    symbol: Atom,
    ids: Vec<Atom>,
    renderer: Arc<dyn Fn(&SymbolRef) -> String + Send + Sync>,
  ) -> Self {
    Self {
      module,
      symbol,
      ids,
      renderer,
    }
  }

  pub fn render(&self) -> String {
    (self.renderer)(self)
  }
}

#[derive(Debug, Clone)]
pub enum ModuleEvaluation {
  Direct(Atom),
  Initializer(Atom),
}

#[derive(Debug, Clone)]
pub struct WrappedInterop {
  pub module: ModuleIdentifier,
  /// How this chunk obtains the module value. This is deliberately separate
  /// from whether the target's code is scope-hoisted or factory-wrapped.
  pub evaluation: Option<ModuleEvaluation>,
  pub from_module: IdentifierSet,
  pub required_symbol: Option<Atom>,
  pub default_access: Option<Atom>,
  pub default_exported: Option<Atom>,
  pub namespace_object: Option<Atom>,
  pub namespace_object2: Option<Atom>,
  pub property_access: FxIndexMap<Atom, Atom>,
}

fn get_or_create_interop_name(
  required_symbol: &mut Option<Atom>,
  field: &mut Option<Atom>,
  suffix: &str,
  used_names: &mut FxHashSet<Atom>,
) -> Atom {
  if required_symbol.is_none() {
    let new_name = find_new_name("", used_names, &[]);
    used_names.insert(new_name.clone());
    *required_symbol = Some(new_name);
  }
  if let Some(existing) = field {
    return existing.clone();
  }
  let mut new_name = Atom::new(format!(
    "{}{}",
    required_symbol.as_ref().expect("already set"),
    suffix
  ));
  if used_names.contains(&new_name) {
    new_name = find_new_name(new_name.as_str(), used_names, &[]);
  }
  *field = Some(new_name.clone());
  used_names.insert(new_name.clone());
  new_name
}

impl WrappedInterop {
  pub fn namespace(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    get_or_create_interop_name(
      &mut self.required_symbol,
      &mut self.namespace_object,
      "_namespace",
      used_names,
    )
  }

  pub fn namespace2(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    get_or_create_interop_name(
      &mut self.required_symbol,
      &mut self.namespace_object2,
      "_namespace2",
      used_names,
    )
  }

  pub fn default_access(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    get_or_create_interop_name(
      &mut self.required_symbol,
      &mut self.default_access,
      "_default",
      used_names,
    )
  }

  pub fn default_exported(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &[]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(default_exported) = &self.default_exported {
      return default_exported.clone();
    }

    let default_access_symbol = self.default_access(used_names);
    let default_exported_symbol = find_new_name(&default_access_symbol, used_names, &[]);
    used_names.insert(default_exported_symbol.clone());
    self.default_exported = Some(default_exported_symbol.clone());
    default_exported_symbol
  }

  pub fn property_access(&mut self, atom: &Atom, used_names: &mut FxHashSet<Atom>) -> Atom {
    self.property_access.get(atom).cloned().unwrap_or_else(|| {
      let local_name = find_new_name(atom, used_names, &[]);
      used_names.insert(local_name.clone());
      self.property_access.insert(atom.clone(), local_name);
      self
        .property_access
        .get(atom)
        .expect("just inserted")
        .clone()
    })
  }

  pub fn render(
    &self,
    compilation: &Compilation,
    runtime_template: &RuntimeCodeTemplate,
  ) -> ConcatSource {
    self.render_with_mode(compilation, runtime_template, true)
  }

  pub fn render_assignments(
    &self,
    compilation: &Compilation,
    runtime_template: &RuntimeCodeTemplate,
  ) -> ConcatSource {
    self.render_with_mode(compilation, runtime_template, false)
  }

  pub fn declaration_names(&self) -> impl Iterator<Item = &Atom> {
    self
      .required_symbol
      .iter()
      .chain(self.namespace_object.iter())
      .chain(self.namespace_object2.iter())
      .chain(self.default_access.iter())
      .chain(self.default_exported.iter())
      .chain(self.property_access.values())
  }

  fn render_with_mode(
    &self,
    compilation: &Compilation,
    runtime_template: &RuntimeCodeTemplate,
    declarations: bool,
  ) -> ConcatSource {
    let mut source = ConcatSource::default();
    let name = self.required_symbol.as_ref();
    let evaluation = self.evaluation.as_ref().unwrap_or_else(|| {
      panic!(
        "module interop {:?} from {:?} should have a linked evaluation plan; required_symbol={:?}",
        self.module, self.from_module, self.required_symbol
      )
    });

    let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, &self.module);
    let (value, await_value) = match evaluation {
      ModuleEvaluation::Direct(value) => (value.to_string(), false),
      ModuleEvaluation::Initializer(initializer) => (format!("{initializer}()"), is_async),
    };

    if let Some(name) = name {
      source.add(RawStringSource::from(format!(
        // this render only happens at top level scope of the chunk
        "{}{name} = {}{value};\n",
        if declarations { "const " } else { "" },
        if await_value { "await " } else { "" },
      )));

      if let Some(namespace_object) = &self.namespace_object {
        source.add(RawStringSource::from(format!(
          "{}{} = /*#__PURE__*/{}({}, 2);\n",
          if declarations { "var " } else { "" },
          namespace_object,
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          name
        )));
      }

      if let Some(namespace_object) = &self.namespace_object2 {
        source.add(RawStringSource::from(format!(
          "{}{} = /*#__PURE__*/{}({});\n",
          if declarations { "var " } else { "" },
          namespace_object,
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          name
        )));
      }

      if let Some(default_access) = &self.default_access {
        source.add(RawStringSource::from(format!(
          "{}{} = /*#__PURE__*/{}({});\n",
          if declarations { "var " } else { "" },
          default_access,
          runtime_template.render_runtime_globals(&RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT),
          name
        )));

        if let Some(default_exported_symbol) = &self.default_exported {
          source.add(RawStringSource::from(format!(
            "{}{default_exported_symbol} = {default_access}();\n",
            if declarations { "var " } else { "" },
          )));
        }
      }

      for (s, local) in &self.property_access {
        source.add(RawStringSource::from(format!(
          "{}{local} = {name}.{s};\n",
          if declarations { "var " } else { "" },
        )));
      }
    } else {
      if matches!(evaluation, ModuleEvaluation::Initializer(_)) {
        source.add(RawStringSource::from(format!(
          "{}{value};\n",
          if await_value { "await " } else { "" },
        )));
      }
    }

    source
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReExportFrom {
  Chunk(ChunkUkey),
  Request(String),
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub enum RawImportSource {
  Chunk(ChunkUkey),
  Source((String, Option<String>)),
}

#[derive(Debug, Clone)]
pub struct ChunkLinkContext {
  pub chunk: ChunkUkey,

  pub decl_before_exports: FxIndexSet<String>,

  /**
  specifier order doesn't matter, we can sort them based on name
  Map<module_id, Map<local_name, export_name>>
  */
  exports: FxHashMap<Atom, FxIndexSet<Atom>>,

  /**
  symbols that this chunk provides
  */
  pub exported_symbols: FxHashSet<Atom>,

  /**
  exports that need to be re-exported
  Map<chunk, Map<local_name, export_name>>
  */
  re_exports: FxIndexMap<ReExportFrom, FxHashMap<Atom, FxHashSet<Atom>>>,

  /**
   * re exports in raw form, used for rendering export * from 'module'
   */
  pub raw_star_exports: FxIndexMap<String, FxIndexSet<Atom>>,

  /**
  import order matters, it affects execution order
  */
  pub imports: IdentifierIndexMap<FxHashMap<Atom, Atom>>,

  /**
  raw import statements
   */
  pub raw_import_stmts: FxIndexMap<RawImportSource, ImportSpec>,

  /**
  namespace imports already provided by module init fragments
   */
  pub module_external_namespace_imports: FxHashMap<RawImportSource, Atom>,

  /**
  `const symbol = __rspack_require(module_id)`
  */
  pub required: IdentifierIndexMap<WrappedInterop>,

  /**
  which module needs namespace objects
  */
  pub needed_namespace_objects: IdentifierIndexSet,

  pub namespace_object_sources: IdentifierMap<String>,

  pub init_fragments: Vec<BoxChunkInitFragment>,

  pub hashbang: Option<String>,

  pub directives: Vec<String>,

  /**
  modules that can be scope hoisted
  */
  pub hoisted_modules: IdentifierIndexSet,

  /**
  modules that needs wrapper
  */
  pub wrapped_modules: IdentifierIndexSet,

  /** Direct initializer declarations and imported aliases visible in this chunk. */
  pub module_initializers: IdentifierMap<Atom>,

  /** ESM export name for each initializer declared by this chunk. */
  pub module_initializer_exports: IdentifierMap<Atom>,

  /** Initializers whose body is scope-hoisted instead of a CommonJS factory. */
  pub hoisted_initializers: IdentifierIndexSet,

  /** Optional namespace export returned after a cross-chunk hoisted initializer runs. */
  pub initializer_namespace_exports: IdentifierMap<Atom>,

  /** Namespace bindings used by wrapped factories to reference hoisted modules. */
  pub hoisted_namespaces: IdentifierMap<Atom>,

  /// Shared esbuild-style helper used to create wrapped initializers.
  pub commonjs_helper: Option<Atom>,

  /// Shared esbuild-style helper used to guard scope-hoisted module bodies.
  pub esm_helper: Option<Atom>,

  /// Deconflicted scratch binding used by async initializers to avoid an
  /// `await` once a dependency initializer has already settled.
  pub async_dependency_temp: Option<Atom>,

  /**
  modules that needs wrapper
  */
  pub refs: FxHashMap<String, Ref>,

  /**
  all used symbols in current chunk
  */
  pub used_names: FxHashSet<Atom>,
}

impl ChunkLinkContext {
  pub fn new(
    chunk_ukey: ChunkUkey,
    hoisted_modules: IdentifierIndexSet,
    wrapped_modules: IdentifierIndexSet,
  ) -> Self {
    ChunkLinkContext {
      chunk: chunk_ukey,
      hoisted_modules,
      wrapped_modules,
      module_initializers: Default::default(),
      module_initializer_exports: Default::default(),
      hoisted_initializers: Default::default(),
      initializer_namespace_exports: Default::default(),
      hoisted_namespaces: Default::default(),
      commonjs_helper: None,
      esm_helper: None,
      async_dependency_temp: None,
      decl_before_exports: Default::default(),
      exports: Default::default(),
      re_exports: Default::default(),
      imports: Default::default(),
      required: Default::default(),
      needed_namespace_objects: Default::default(),
      namespace_object_sources: Default::default(),
      init_fragments: Default::default(),
      hashbang: None,
      directives: Default::default(),
      refs: Default::default(),
      used_names: Default::default(),
      exported_symbols: Default::default(),
      raw_import_stmts: Default::default(),
      module_external_namespace_imports: Default::default(),
      raw_star_exports: Default::default(),
    }
  }

  pub fn exports(&self) -> &FxHashMap<Atom, FxIndexSet<Atom>> {
    &self.exports
  }

  pub fn exports_mut(&mut self) -> &mut FxHashMap<Atom, FxIndexSet<Atom>> {
    &mut self.exports
  }

  pub fn re_exports(&self) -> &FxIndexMap<ReExportFrom, FxHashMap<Atom, FxHashSet<Atom>>> {
    &self.re_exports
  }

  pub fn re_exports_mut(
    &mut self,
  ) -> &mut FxIndexMap<ReExportFrom, FxHashMap<Atom, FxHashSet<Atom>>> {
    &mut self.re_exports
  }
}
