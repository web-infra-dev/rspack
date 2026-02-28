use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxChunkInitFragment, ChunkGraph, ChunkUkey, Compilation, ImportSpec, ModuleGraph,
  ModuleIdentifier, RuntimeCodeTemplate, RuntimeGlobals, find_new_name,
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
pub struct ExternalInterop {
  pub module: ModuleIdentifier,
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

impl ExternalInterop {
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
    runtime_template: &RuntimeCodeTemplate<'_>,
  ) -> ConcatSource {
    let mut source = ConcatSource::default();
    let name = self.required_symbol.as_ref();

    let is_async = ModuleGraph::is_async(&compilation.async_modules_artifact, &self.module);

    if let Some(name) = name {
      source.add(RawStringSource::from(format!(
        // this render only happens at top level scope of the chunk
        "const {name} = {}{}({});\n",
        if is_async { "await " } else { "" },
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        serde_json::to_string(
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, self.module)
            .unwrap_or_else(|| panic!("should set module id for {:?}", self.module))
            .as_str()
        )
        .expect("module id to string should success")
      )));

      if let Some(namespace_object) = &self.namespace_object {
        source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({}, 2);\n",
          namespace_object,
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          name
        )));
      }

      if let Some(namespace_object) = &self.namespace_object2 {
        source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({});\n",
          namespace_object,
          runtime_template.render_runtime_globals(&RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT),
          name
        )));
      }

      if let Some(default_access) = &self.default_access {
        source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({});\n",
          default_access,
          runtime_template.render_runtime_globals(&RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT),
          name
        )));

        if let Some(default_exported_symbol) = &self.default_exported {
          source.add(RawStringSource::from(format!(
            "var {default_exported_symbol} = {default_access}();\n",
          )));
        }
      }

      for (s, local) in &self.property_access {
        source.add(RawStringSource::from(format!(
          "var {local} = {name}.{s};\n"
        )));
      }
    } else {
      source.add(RawStringSource::from(format!(
        "{}({});\n",
        runtime_template.render_runtime_globals(&RuntimeGlobals::REQUIRE),
        serde_json::to_string(
          ChunkGraph::get_module_id(&compilation.module_ids_artifact, self.module)
            .unwrap_or_else(|| panic!("should set module id for {}", self.module))
            .as_str()
        )
        .expect("module id to string should success")
      )));
    }

    source
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReExportFrom {
  Chunk(ChunkUkey),
  Request(String),
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
  pub raw_import_stmts: FxIndexMap<(String, Option<String>), ImportSpec>,

  /**
  `const symbol = __webpack_require__(module_id)`
  */
  pub required: IdentifierIndexMap<ExternalInterop>,

  /**
  which module needs namespace objects
  */
  pub needed_namespace_objects: IdentifierIndexSet,

  pub namespace_object_sources: IdentifierMap<String>,

  pub init_fragments: Vec<BoxChunkInitFragment>,

  /**
  modules that can be scope hoisted
  */
  pub hoisted_modules: IdentifierIndexSet,

  /**
  modules that needs wrapper
  */
  pub decl_modules: IdentifierIndexSet,

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
    decl_modules: IdentifierIndexSet,
  ) -> Self {
    ChunkLinkContext {
      chunk: chunk_ukey,
      hoisted_modules,
      decl_modules,
      decl_before_exports: Default::default(),
      exports: Default::default(),
      re_exports: Default::default(),
      imports: Default::default(),
      required: Default::default(),
      needed_namespace_objects: Default::default(),
      namespace_object_sources: Default::default(),
      init_fragments: Default::default(),
      refs: Default::default(),
      used_names: Default::default(),
      exported_symbols: Default::default(),
      raw_import_stmts: Default::default(),
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
