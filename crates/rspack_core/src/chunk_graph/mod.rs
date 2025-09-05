use core::fmt;
use std::{borrow::Cow, sync::Arc};

use itertools::Itertools;
use rspack_collections::{
  IdentifierIndexMap, IdentifierIndexSet, IdentifierMap, IdentifierSet, UkeyMap,
};
use rspack_sources::{ConcatSource, RawStringSource};
use rspack_util::{atom::Atom, env::has_query, fx_hash::FxIndexMap};
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

use crate::{
  AsyncDependenciesBlockIdentifier, ChunkGroupUkey, ChunkUkey, Compilation, ModuleIdentifier,
  RuntimeGlobals, find_new_name,
};

pub mod chunk_graph_chunk;
pub mod chunk_graph_module;
pub use chunk_graph_chunk::{ChunkGraphChunk, ChunkSizeOptions};
pub use chunk_graph_module::{ChunkGraphModule, ModuleId};

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

pub trait BindingRenderer {
  fn render<'a>(&'a self) -> Cow<'a, str>;
}

#[derive(Debug, Clone, Default)]
pub struct ExternalInterop {
  pub module: ModuleIdentifier,
  pub from_module: IdentifierSet,
  pub required_symbol: Option<Atom>,
  pub default_access: Option<Atom>,
  pub namespace_object: Option<Atom>,
  pub namespace_object2: Option<Atom>,
  pub property_access: FxIndexMap<Atom, Atom>,
}

impl ExternalInterop {
  pub fn namespace(&mut self, used_names: &mut HashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(namespace_object) = &self.namespace_object {
      namespace_object.clone()
    } else {
      let mut new_name = format!(
        "{}_namespace",
        self.required_symbol.as_ref().expect("already set")
      )
      .into();

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }
      self.namespace_object = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name
    }
  }

  pub fn namespace2(&mut self, used_names: &mut HashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(namespace_object) = &self.namespace_object2 {
      namespace_object.clone()
    } else {
      let mut new_name = format!(
        "{}_namespace2",
        self.required_symbol.as_ref().expect("already set")
      )
      .into();

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }
      self.namespace_object2 = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name
    }
  }

  pub fn default_access(&mut self, used_names: &mut HashSet<Atom>) -> Atom {
    if self.required_symbol.is_none() {
      let new_name = find_new_name("", used_names, &vec![]);
      used_names.insert(new_name.clone());
      self.required_symbol = Some(new_name);
    }

    if let Some(default_access) = &self.default_access {
      default_access.clone()
    } else {
      let mut new_name = format!(
        "{}_default",
        self.required_symbol.as_ref().expect("already set")
      )
      .into();

      if used_names.contains(&new_name) {
        new_name = find_new_name(new_name.as_str(), used_names, &vec![]);
      }

      self.default_access = Some(new_name.clone());
      used_names.insert(new_name.clone());
      new_name.clone()
    }
  }

  pub fn property_access(&mut self, atom: &Atom, used_names: &mut HashSet<Atom>) -> Atom {
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
            .expect("should set module id")
            .as_str()
        )
        .expect("module id to string should success")
      )));
    }

    source
  }
}

#[derive(Debug, Clone)]
pub struct ChunkLinkContext {
  pub chunk: ChunkUkey,

  // specifier order doesn't matter, we can sort them based on name
  // Map<module_id, Map<local_name, export_name>>
  exports: IdentifierMap<HashMap<Atom, Atom>>,

  // exports that need to be re-exported
  // Map<chunk, Map<local_name, export_name>>
  re_exports: UkeyMap<ChunkUkey, HashMap<Atom, Atom>>,

  // import order matters, it affects execution order
  pub imports: IdentifierIndexMap<HashMap<Atom, Atom>>,

  // const symbol = __webpack_require__(module_id)
  pub required: IdentifierIndexMap<ExternalInterop>,

  // which module needs namespace objects
  pub needed_namespace_objects: IdentifierIndexSet,

  pub namespace_object_sources: IdentifierMap<String>,

  // modules that can be scope hoisted
  pub hoisted_modules: IdentifierIndexSet,

  // modules that needs wrapper
  pub decl_modules: IdentifierIndexSet,

  // modules that needs wrapper
  pub refs: HashMap<String, Ref>,

  // Map::<module, (is_module_in_chunk, symbol_binding)>
  pub dyn_refs: HashMap<String, (bool, Ref)>,

  // all used symbols in current chunk
  pub used_names: HashSet<Atom>,
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
      refs: Default::default(),
      dyn_refs: Default::default(),
      used_names: Default::default(),
    }
  }

  pub fn add_export(
    &mut self,
    module_id: ModuleIdentifier,
    local_name: Atom,
    export_name: Atom,
  ) -> &mut Atom {
    self
      .exports
      .entry(module_id)
      .or_default()
      .entry(local_name)
      .or_insert(export_name)
  }

  pub fn add_re_export(
    &mut self,
    chunk: ChunkUkey,
    local_name: Atom,
    export_name: Atom,
  ) -> &mut Atom {
    self
      .re_exports
      .entry(chunk)
      .or_default()
      .entry(local_name)
      .or_insert(export_name)
  }

  pub fn exports(&self) -> &IdentifierMap<HashMap<Atom, Atom>> {
    &self.exports
  }

  pub fn exports_mut(&mut self) -> &mut IdentifierMap<HashMap<Atom, Atom>> {
    &mut self.exports
  }

  pub fn re_exports(&self) -> &UkeyMap<ChunkUkey, HashMap<Atom, Atom>> {
    &self.re_exports
  }
}

#[derive(Debug, Clone, Default)]
pub struct ChunkGraph {
  /// If a module is imported dynamically, it will be assigned to a unique ChunkGroup
  pub(crate) block_to_chunk_group_ukey: HashMap<AsyncDependenciesBlockIdentifier, ChunkGroupUkey>,

  pub(crate) chunk_graph_module_by_module_identifier: IdentifierMap<ChunkGraphModule>,
  chunk_graph_chunk_by_chunk_ukey: UkeyMap<ChunkUkey, ChunkGraphChunk>,

  runtime_ids: HashMap<String, Option<String>>,

  // only used for esm output
  pub link: Option<UkeyMap<ChunkUkey, ChunkLinkContext>>,
}

impl ChunkGraph {
  pub fn is_entry_module(&self, module_id: &ModuleIdentifier) -> bool {
    let cgm = self.expect_chunk_graph_module(*module_id);
    !cgm.entry_in_chunks.is_empty()
  }
}
static INDENT: &str = "    ";

impl ChunkGraph {
  // convert chunk graph to dot format
  // 1. support chunk_group dump visualizer
  pub fn to_dot(&self, compilation: &Compilation) -> std::result::Result<String, fmt::Error> {
    let mut visited_group_nodes: HashMap<ChunkGroupUkey, String> = HashMap::default();
    let mut visited_group_edges: HashSet<(ChunkGroupUkey, ChunkGroupUkey, bool)> =
      HashSet::default();
    let mut visiting_groups: Vec<ChunkGroupUkey> = Vec::new();
    let module_graph = compilation.get_module_graph();
    // generate following chunk_group_info as dto record info
    // <td title="chunk_group_name"></td><td title="chunk1"></td><td title="chunk2"></td>
    let get_debug_chunk_group_info = |chunk_group_ukey: &ChunkGroupUkey| {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(chunk_group_ukey)
        .expect("should have chunk group");

      let chunk_group_name = chunk_group.name().map_or_else(
        || {
          let mut origins = chunk_group
            .origins()
            .iter()
            .filter_map(|record| {
              record.request.as_deref().and_then(|request| {
                record.module.as_ref().map(|module_id| {
                  (
                    module_graph
                      .module_by_identifier(module_id)
                      .expect("should have module")
                      .readable_identifier(&compilation.options.context),
                    request,
                  )
                })
              })
            })
            .map(|(module, request)| format!("{module} {request}"))
            .collect::<Vec<_>>();

          origins.sort();
          Cow::Owned(origins.join("\n"))
        },
        Cow::Borrowed,
      );
      let table_header = format!("<tr><td bgcolor=\"#aaa\">{chunk_group_name}</td></tr>");
      let bg_color = if chunk_group.is_initial() {
        "green"
      } else {
        "orange"
      };

      let requests = chunk_group
        .chunks
        .iter()
        .map(|chunk_ukey| {
          let chunk: &crate::Chunk = compilation
            .chunk_by_ukey
            .get(chunk_ukey)
            .expect("should have chunk");
          if let Some(name) = chunk.name() {
            return name.to_string();
          }

          chunk_ukey.as_u32().to_string()
        })
        .map(|chunk_name| format!("    <tr><td>{chunk_name}</td></tr>"))
        .join("\n");

      let table_body = requests.to_string();

      format!("\n<<table bgcolor=\"{bg_color}\">\n{table_header}\n{table_body}\n</table>>\n")
    };

    // push entry_point chunk group into visiting queue
    for (_, chunk_group_ukey) in compilation.entrypoints() {
      visiting_groups.push(*chunk_group_ukey);
    }
    // bfs visit all chunk groups
    while let Some(chunk_group_ukey) = visiting_groups.pop() {
      let chunk_group = compilation
        .chunk_group_by_ukey
        .get(&chunk_group_ukey)
        .expect("should have chunk group");
      if visited_group_nodes.contains_key(&chunk_group_ukey) {
        continue;
      }
      let chunk_group_name = get_debug_chunk_group_info(&chunk_group_ukey);

      for parent in &chunk_group.parents {
        // false means this is a revert link to parent
        visited_group_edges.insert((chunk_group_ukey, *parent, false));
      }
      for child in chunk_group.children.iter() {
        // calculate every edge
        visited_group_edges.insert((chunk_group_ukey, *child, true));
        visiting_groups.push(*child);
      }
      visited_group_nodes.insert(chunk_group_ukey, chunk_group_name.clone());
    }
    use std::fmt::Write;
    let mut dot = String::new();
    // write header
    writeln!(&mut dot, "digraph G {{")?;
    // neato layout engine is more readable
    writeln!(&mut dot, "layout=neato;")?;
    writeln!(&mut dot, "overlap=false;")?;
    writeln!(&mut dot, "node [shape=plaintext];")?;
    writeln!(&mut dot, "edge [arrowsize=0.5];")?;

    // write all node info
    for (node_id, node_info) in visited_group_nodes.iter() {
      writeln!(&mut dot, "{} {} [", INDENT, node_id.as_u32())?;
      write!(&mut dot, "label={node_info}")?;
      write!(&mut dot, "\n];\n")?;
    }
    // write all edge info
    // 1 -> 2, 2 -> 3
    for edge in visited_group_edges.iter() {
      write!(&mut dot, "{} -> {}", edge.0.as_u32(), edge.1.as_u32())?;
      write!(&mut dot, "[")?;
      write!(
        &mut dot,
        "style=\"{}\"",
        if edge.2 { "solid" } else { "dotted" }
      )?;
      write!(&mut dot, "]")?;
      writeln!(&mut dot, ";")?;
    }
    // write footer
    write!(&mut dot, "}}")?;
    Ok(dot)
  }
  pub async fn generate_dot(&self, compilation: &Compilation, dotfile_name: &str) {
    // do not generate dot file if there is no query
    if !has_query() {
      return;
    }
    let result = self.to_dot(compilation).expect("to_dot failed");
    compilation
      .output_filesystem
      .write(
        format!(
          "{}-{}.dot",
          compilation.compiler_id().as_u32(),
          dotfile_name
        )
        .as_str()
        .into(),
        result.as_bytes(),
      )
      .await
      .expect("write dot file failed");
  }
}
