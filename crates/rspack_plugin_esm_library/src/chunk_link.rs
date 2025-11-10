use std::{borrow::Cow, sync::Arc};

use rspack_collections::{IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet};
use rspack_core::{
  BoxChunkInitFragment, ChunkGraph, ChunkUkey, Compilation, ImportSpec, ModuleIdentifier,
  RuntimeGlobals, find_new_name,
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

impl ExternalInterop {
  pub fn namespace(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(namespace_object) = &self.namespace_object {
      namespace_object.clone()
    } else {
      let mut new_name = Atom::new(format!(
        "{}_namespace",
        self.required_symbol.as_ref().expect("already set")
      ));

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }
      self.namespace_object = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name
    }
  }

  pub fn namespace2(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(namespace_object) = &self.namespace_object2 {
      namespace_object.clone()
    } else {
      let mut new_name = Atom::new(format!(
        "{}_namespace2",
        self.required_symbol.as_ref().expect("already set")
      ));

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }
      self.namespace_object2 = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name
    }
  }

  pub fn default_access(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(default_access) = &self.default_access {
      default_access.clone()
    } else {
      let mut new_name = Atom::new(format!(
        "{}_default",
        self.required_symbol.as_ref().expect("already set")
      ));

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }

      self.default_access = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name.clone()
    }
  }

  pub fn default_exported(&mut self, used_names: &mut FxHashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(default_exported) = &self.default_exported {
      return default_exported.clone();
    }

    let default_access_symbol = self.default_access(used_names);
    let default_exported_symbol = find_new_name(&default_access_symbol, used_names, &vec![]);
    used_names.insert(default_exported_symbol.clone());
    self.default_exported = Some(default_exported_symbol.clone());
    default_exported_symbol
  }

  pub fn property_access(&mut self, atom: &Atom, used_names: &mut FxHashSet<Atom>) -> Atom {
    self.property_access.get(atom).cloned().unwrap_or_else(|| {
      let local_name = find_new_name(atom, used_names, &vec![]);
      self
        .property_access
        .insert(atom.clone(), local_name.clone());
      self
        .property_access
        .get(atom)
        .expect("just inserted")
        .clone()
    })
  }

  pub fn render(&self, compilation: &Compilation) -> ConcatSource {
    let mut source = ConcatSource::default();

    let name = self.required_symbol.as_ref();
    if let Some(name) = name {
      source.add(RawStringSource::from(format!(
        "const {name} = {}({});\n",
        RuntimeGlobals::REQUIRE,
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
          RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
          name
        )));
      }

      if let Some(namespace_object) = &self.namespace_object2 {
        source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({});\n",
          namespace_object,
          RuntimeGlobals::CREATE_FAKE_NAMESPACE_OBJECT,
          name
        )));
      }

      if let Some(default_access) = &self.default_access {
        source.add(RawStringSource::from(format!(
          "var {} = /*#__PURE__*/{}({});\n",
          default_access,
          RuntimeGlobals::COMPAT_GET_DEFAULT_EXPORT,
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
        RuntimeGlobals::REQUIRE,
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
  const symbol = __webpack_require__(module_id)
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
  Map::<module, (is_module_in_chunk, symbol_binding)>
  */
  pub dyn_refs: FxHashMap<String, (bool, Ref)>,

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
      exports: Default::default(),
      re_exports: Default::default(),
      imports: Default::default(),
      required: Default::default(),
      needed_namespace_objects: Default::default(),
      namespace_object_sources: Default::default(),
      init_fragments: Default::default(),
      refs: Default::default(),
      dyn_refs: Default::default(),
      used_names: Default::default(),
      exported_symbols: Default::default(),
      raw_import_stmts: Default::default(),
      raw_star_exports: Default::default(),
    }
  }

  pub fn add_export(&mut self, local_name: Atom, export_name: Atom) -> &Atom {
    let exported = if self.exported_symbols.insert(export_name.clone()) {
      export_name
    } else {
      let new_name = find_new_name(&local_name, &self.used_names, &vec![]);
      self.exported_symbols.insert(new_name.clone());
      self.used_names.insert(new_name.clone());
      new_name
    };

    let set = self.exports.entry(local_name.clone()).or_default();
    set.insert(exported.clone());
    set.get(&exported).expect("just inserted")
  }

  pub fn add_re_export_from_request(
    &mut self,
    request: String,
    imported_name: Atom,
    export_name: Atom,
  ) {
    self.exported_symbols.insert(export_name.clone());

    self
      .re_exports
      .entry(ReExportFrom::Request(request))
      .or_default()
      .entry(imported_name)
      .or_default()
      .insert(export_name);
  }

  pub fn add_re_export(&mut self, chunk: ChunkUkey, local_name: Atom, export_name: Atom) -> &Atom {
    let export_name = if self.exported_symbols.insert(export_name.clone()) {
      export_name
    } else {
      let new_name = find_new_name(&local_name, &self.used_names, &vec![]);
      self.used_names.insert(new_name.clone());
      self.exported_symbols.insert(new_name.clone());
      new_name
    };

    let set = self
      .re_exports
      .entry(ReExportFrom::Chunk(chunk))
      .or_default()
      .entry(local_name)
      .or_default();

    set.insert(export_name.clone());
    set.get(&export_name).expect("should have inserted")
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
}
