use std::sync::atomic::{AtomicUsize, Ordering};

use rayon::prelude::*;
use rspack_cacheable::{ContextIdentifierLayout, cacheable, utils::OwnedOrRef};
use rspack_collections::IdentifierSet;
use rspack_error::{Result, error};
use rustc_hash::{FxHashMap, FxHashSet};

use super::{
  super::BoundEntry,
  alternatives::{TempDependency, TempModule},
};
use crate::{
  AsyncDependenciesBlockIdentifier, AsyncDependenciesBlockRef, BoxDependency, CompilationLogger,
  ContextElementDependency, DependenciesBlock, Dependency, DependencyId, DependencyParents,
  DependencyRef, FactorizationArtifact, FactorizeInfo, ModuleGraph, ModuleGraphConnection,
  ModuleGraphModule, ModuleIdentifier, ModuleIssuer, ModuleRef, RayonConsumer,
  cache::CacheCodec,
  compilation::build_module_graph::{LazyDependencies, ModuleToLazyMake},
  legacy_cache::persistent::storage::Storage,
};

pub const SCOPE: &str = "occasion_make_module_graph";

type CachedDependency<'a> = (
  OwnedOrRef<'a, DependencyRef>,
  Option<OwnedOrRef<'a, AsyncDependenciesBlockIdentifier>>,
  usize,
);

/// Build-owned inputs, without factorization results or graph edges.
#[cacheable]
struct Node<'a> {
  pub mgm: OwnedOrRef<'a, ModuleGraphModule>,
  pub module: OwnedOrRef<'a, ModuleRef>,
  pub dependencies: Vec<CachedDependency<'a>>,
  pub blocks: Vec<OwnedOrRef<'a, AsyncDependenciesBlockRef>>,
  pub lazy_info: Option<OwnedOrRef<'a, LazyDependencies>>,
}

#[cacheable]
struct Resolution<'a> {
  /// Factory replacements only. Unchanged inputs must keep sharing the build's dependency
  /// objects, including mutable lazy flags, after recovery.
  dependencies: Vec<OwnedOrRef<'a, DependencyRef>>,
  infos: Vec<OwnedOrRef<'a, FactorizeInfo>>,
  connections: Vec<OwnedOrRef<'a, ModuleGraphConnection>>,
}

#[cacheable]
struct ResolutionSegment {
  dependencies: Vec<DependencyId>,
  bytes: Vec<u8>,
}

/// One storage value commits all segments together. This framing is encoded directly, NOT
/// through BoundEntry: scanning the concatenated bytes would bind the build to resolver paths.
/// Each segment has its own binding envelope and checked archive. A malformed frame or accepted
/// segment fails recovery of the occasion; only a different bound root is a repairable miss.
#[cacheable]
struct SegmentedNode {
  build: Vec<u8>,
  resolutions: Vec<ResolutionSegment>,
  issuer: Vec<u8>,
}

pub struct RecoveredModuleGraph {
  pub mg: ModuleGraph,
  pub factorization_artifact: FactorizationArtifact,
  pub module_to_lazy_make: ModuleToLazyMake,
  pub entry_dependencies: FxHashSet<DependencyId>,
  /// Dependencies whose resolution expired or target was dropped. The next update refactorizes
  /// them while their parent module keeps its cached build.
  pub refactorize_dependencies: FxHashSet<DependencyId>,
  pub check_issuer_cycles: bool,
  /// Original storage keys of nodes rejected by root binding or key validation.
  pub dropped: Vec<Vec<u8>>,
}

/// Layouts of the identifiers that embed a context directory or a context regex.
fn context_identifier_layouts(mg: &ModuleGraph) -> FxHashMap<String, ContextIdentifierLayout> {
  let module_regex = |identifier: &ModuleIdentifier| {
    mg.module_by_identifier(identifier)?
      .as_context_module()?
      .identifier_regex_ranges()
      .into()
  };
  let mut layouts: FxHashMap<String, ContextIdentifierLayout> = FxHashMap::default();
  for (identifier, _) in mg.modules() {
    if let Some(ranges) = module_regex(identifier) {
      layouts.entry(identifier.to_string()).or_default().regexes = ranges;
    }
  }
  for (_, dependency) in mg.dependencies() {
    if let Some(dependency) = dependency.as_context_dependency() {
      layouts
        .entry(crate::ContextDependency::resource_identifier(dependency).to_string())
        .or_default()
        .regexes = dependency.resource_identifier_regex_ranges();
    }
    if let Some(dependency) = dependency.downcast_ref::<ContextElementDependency>()
      && let Some(offset) = dependency.resource_identifier_context_offset()
    {
      layouts
        .entry(dependency.resource_identifier.to_string())
        .or_default()
        .paths = vec![offset];
    }
  }
  for (identifier, block) in mg.blocks() {
    let paths = block.context_element_offsets();
    let regexes = module_regex(block.parent()).unwrap_or_default();
    if paths.is_empty() && regexes.is_empty() {
      continue;
    }
    layouts.insert(
      identifier.as_str().to_string(),
      ContextIdentifierLayout { regexes, paths },
    );
  }
  layouts
}

fn valid_block_layout(block: &(impl DependenciesBlock + ?Sized)) -> bool {
  block.get_blocks().iter().copied().eq(
    block
      .get_block_refs()
      .iter()
      .map(|block| block.identifier()),
  )
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::save", skip_all)]
pub fn save_module_graph(
  mg: &ModuleGraph,
  factorization_artifact: &FactorizationArtifact,
  module_to_lazy_make: &ModuleToLazyMake,
  removed_modules: &IdentifierSet,
  need_update_modules: &IdentifierSet,
  storage: &mut dyn Storage,
  codec: &CacheCodec,
) {
  for identifier in removed_modules {
    storage.remove(
      SCOPE,
      &codec.encode(identifier).expect("should encode identifier"),
    );
  }

  let node_codec = if codec.is_portable() && !need_update_modules.is_empty() {
    Some(codec.with_context_identifier_layouts(context_identifier_layouts(mg)))
  } else {
    None
  };
  let node_codec = node_codec.as_ref().unwrap_or(codec);

  // save module_graph
  let saved_count = AtomicUsize::new(0);
  need_update_modules
    .par_iter()
    .map(|identifier| {
      let mgm = mg
        .module_graph_module_by_identifier(identifier)
        .expect("should have mgm");
      let module = mg
        .module_by_identifier(identifier)
        .expect("should have module");
      let blocks = module
        .get_blocks()
        .par_iter()
        .map(|block_id| {
          mg.block_ref_by_id(block_id)
            .expect("should have block")
            .into()
        })
        .collect::<Vec<_>>();
      // Factories may replace graph dependencies (for example a resolved mock request). The
      // module and its blocks retain build inputs, which must be used when resolving again.
      let mut build_dependencies = FxHashMap::default();
      for dep in module.get_dependencies() {
        build_dependencies.insert(*dep.id(), dep);
      }
      let mut queue = module.get_block_refs().iter().collect::<Vec<_>>();
      while let Some(block) = queue.pop() {
        for dep in block.get_dependencies() {
          build_dependencies.insert(*dep.id(), dep);
        }
        queue.extend(block.get_block_refs());
      }
      let dependencies = mgm
        .all_dependencies()
        .par_iter()
        .map(|dep_id| {
          (
            build_dependencies
              .get(dep_id)
              .copied()
              .unwrap_or_else(|| mg.dependency_ref_by_id(dep_id))
              .into(),
            mg.get_parent_block(dep_id).map(Into::into),
            mg.get_parent_block_index(dep_id).unwrap_or_default(),
          )
        })
        .collect::<Vec<_>>();
      let connections = mgm
        .outgoing_connections()
        .par_iter()
        .map(|connection_id| {
          let con = mg
            .connection_by_id(connection_id)
            .expect("should have connection");
          (con.dependency_id, con)
        })
        .collect::<FxHashMap<_, _>>();
      let lazy_info = module_to_lazy_make
        .get_lazy_dependencies(identifier)
        .map(|lazy_deps| lazy_deps.into());
      // Issuers and outgoing connections describe resolution, not the cached module build.
      let mut build_mgm = mgm.clone();
      build_mgm.set_issuer(ModuleIssuer::Unset);
      for id in mgm.outgoing_connections() {
        build_mgm.remove_outgoing_connection(id);
      }
      let mut groups = FxHashMap::<Option<DependencyId>, Vec<DependencyId>>::default();
      for id in mgm.all_dependencies() {
        let owner = factorization_artifact
          .get(id)
          .map_or(*id, FactorizeInfo::owner_dep_id);
        groups.entry(Some(owner)).or_default().push(*id);
      }
      if !codec.is_portable() {
        groups.clear();
        groups.insert(None, mgm.all_dependencies().to_vec());
      }
      let resolutions = groups
        .into_iter()
        .map(|(owner, ids)| {
          let resolution = Resolution {
            dependencies: ids
              .iter()
              .filter_map(|id| {
                let dep = mg.dependency_ref_by_id(id);
                let unchanged = build_dependencies.get(id).is_some_and(|input| {
                  let input: &DependencyRef = input;
                  std::ptr::addr_eq(input.as_ref(), dep.as_ref())
                });
                (!unchanged).then(|| dep.into())
              })
              .collect(),
            infos: if codec.is_portable() {
              factorization_artifact
                .get_by_owner(owner.as_ref().expect("portable resolution has owner"))
                .map(Into::into)
                .into_iter()
                .collect()
            } else {
              ids
                .iter()
                .filter_map(|id| factorization_artifact.get_by_owner(id).map(Into::into))
                .collect()
            },
            connections: ids
              .iter()
              .filter_map(|id| connections.get(id).map(|con| (*con).into()))
              .collect(),
          };
          (ids, resolution)
        })
        .collect::<Vec<_>>();
      let mut node = Node {
        mgm: build_mgm.into(),
        module: module.into(),
        dependencies,
        blocks,
        lazy_info,
      };
      // Keys use the plain codec, like removal, which cannot see the layout of a removed module.
      let key = codec.encode(identifier).expect("should encode identifier");
      // Original requests remain build inputs. An absolute request can still bind this
      // segment (for example /fixed.js on POSIX); only resolver-owned state was split off.
      let root_sensitive = module.build_info().root_sensitive;
      let encode = |node: &Node<'_>| {
        if codec.is_portable() {
          BoundEntry::encode(node_codec, node, root_sensitive)
        } else {
          codec.encode(&(
            OwnedOrRef::<Node>::from(node),
            OwnedOrRef::<Resolution>::from(&resolutions[0].1),
            OwnedOrRef::<ModuleIssuer>::from(mgm.issuer()),
          ))
        }
      };
      let build = match encode(&node) {
        Ok(bytes) => Ok(bytes),
        Err(err) if err.to_string().contains("unsupported field") => {
          tracing::warn!("to bytes failed {:?}", err);
          // try use alternatives
          node.module = TempModule::transform_from(node.module, Vec::new());
          node.dependencies = node
            .dependencies
            .into_iter()
            .map(|(dep, _, index)| (TempDependency::transform_from(dep), None, index))
            .collect();
          node.blocks = vec![];
          encode(&node)
        }
        Err(err) => Err(err),
      };
      let bytes = if !codec.is_portable() {
        build
      } else {
        build.and_then(|build| {
          let resolutions = resolutions
            .into_iter()
            .map(|(dependencies, resolution)| {
              BoundEntry::encode(node_codec, &resolution, false).map(|bytes| ResolutionSegment {
                dependencies,
                bytes,
              })
            })
            .collect::<Result<Vec<_>>>()?;
          let issuer = BoundEntry::encode(node_codec, mgm.issuer(), false)?;
          codec.encode(&SegmentedNode {
            build,
            resolutions,
            issuer,
          })
        })
      };
      (key, bytes)
    })
    .consume(|(id, bytes)| match bytes {
      Ok(bytes) => {
        storage.set(SCOPE, id, bytes);
        saved_count.fetch_add(1, Ordering::Relaxed);
      }
      Err(err) => {
        // Never leave an older resolution paired with a newer module build.
        tracing::warn!("make persistent cache encode failed: {err}");
        storage.remove(SCOPE, &id);
      }
    });

  tracing::debug!("save {} modules", saved_count.load(Ordering::Relaxed));
}

#[tracing::instrument("Cache::Occasion::Make::ModuleGraph::recovery", skip_all)]
pub async fn recovery_module_graph(
  storage: &dyn Storage,
  codec: &CacheCodec,
  _logger: &CompilationLogger,
) -> Result<RecoveredModuleGraph> {
  let mut mg = ModuleGraph::default();
  let mut factorization_artifact = FactorizationArtifact::default();
  let mut module_to_lazy_make = ModuleToLazyMake::default();
  let entries = storage.load(SCOPE).await?;
  let mut counts = FxHashMap::<ModuleIdentifier, usize>::default();
  let entries = entries
    .into_iter()
    .map(|(key, value)| {
      let identifier = codec.decode::<ModuleIdentifier>(&key)?;
      *counts.entry(identifier).or_default() += 1;
      Ok((identifier, key, value))
    })
    .collect::<Result<Vec<_>>>()?;
  let decoded = entries
    .into_par_iter()
    .map(|(identifier, key, value)| -> Result<_> {
      if counts.get(&identifier).copied().unwrap_or_default() != 1 {
        if !codec.is_portable() {
          return Err(error!("duplicate make cache module identifier"));
        }
        return Ok((key, None));
      }
      if !codec.is_portable() {
        let (node, resolution, issuer) = codec.decode::<(
          OwnedOrRef<Node>,
          OwnedOrRef<Resolution>,
          OwnedOrRef<ModuleIssuer>,
        )>(&value)?;
        let (node, resolution, issuer) = (
          node.into_owned(),
          resolution.into_owned(),
          issuer.into_owned(),
        );
        if identifier != node.module.as_ref().identifier()
          || identifier != node.mgm.as_ref().module_identifier
        {
          return Err(error!("inconsistent make cache module identifier"));
        }
        return Ok((
          key,
          Some((node, vec![(vec![], Some(resolution))], Some(issuer))),
        ));
      }
      let frame = codec.decode::<SegmentedNode>(&value)?;
      let Some(node) = BoundEntry::decode::<Node>(codec, &frame.build)? else {
        return Ok((key, None));
      };
      if identifier != node.module.as_ref().identifier()
        || identifier != node.mgm.as_ref().module_identifier
        || codec.encode(&identifier)? != key
      {
        return Ok((key, None));
      }
      let resolutions = frame
        .resolutions
        .into_iter()
        .map(|segment| {
          let resolution = BoundEntry::decode::<Resolution>(codec, &segment.bytes)?;
          Ok((segment.dependencies, resolution))
        })
        .collect::<Result<Vec<_>>>()?;
      let issuer = BoundEntry::decode::<ModuleIssuer>(codec, &frame.issuer)?;
      Ok((key, Some((node, resolutions, issuer))))
    })
    .collect::<Result<Vec<_>>>()?;
  let mut dropped = vec![];
  let mut nodes = vec![];
  for (key, node) in decoded {
    if let Some(node) = node {
      nodes.push(node);
    } else {
      dropped.push(key);
    }
  }
  let recovered = nodes
    .iter()
    .map(|(node, _, _)| node.module.as_ref().identifier())
    .collect::<IdentifierSet>();
  let mut connections = vec![];
  let mut refactorize_dependencies = FxHashSet::default();
  let mut check_issuer_cycles = !dropped.is_empty();
  let mut dependency_ids = FxHashSet::default();
  let mut block_ids = FxHashSet::default();
  for (node, mut resolutions, issuer) in nodes {
    let mut mgm = node.mgm.into_owned();
    let mut module = node.module.into_owned();
    let mid = module.identifier();
    if !codec.is_portable() {
      // Local caches cannot lose individual segments to root binding. Restore the single
      // archive directly, without building the indexes used to validate partial recovery.
      mgm.set_issuer(issuer.expect("local cache includes issuer"));
      let resolution = resolutions
        .pop()
        .expect("local cache includes resolution")
        .1
        .expect("local resolution cannot expire");
      let mut replacements = resolution
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .map(|dep| (*dep.id(), dep))
        .collect::<FxHashMap<_, _>>();
      for (dep, parent_block, index_in_block) in node.dependencies {
        let input = dep.into_owned();
        let dep = replacements.remove(input.id()).unwrap_or(input);
        mg.set_parents(
          *dep.id(),
          DependencyParents {
            block: parent_block.map(OwnedOrRef::into_owned),
            module: mid,
            index_in_block,
          },
        );
        mg.add_dependency_ref(dep);
      }
      for info in resolution.infos.into_iter().map(OwnedOrRef::into_owned) {
        factorization_artifact.insert(info);
      }
      connections.extend(
        resolution
          .connections
          .into_iter()
          .map(OwnedOrRef::into_owned),
      );
      let mut blocks = node
        .blocks
        .into_iter()
        .map(OwnedOrRef::into_owned)
        .collect::<Vec<_>>();
      while let Some(block) = blocks.pop() {
        blocks.extend(block.get_block_refs().iter().cloned());
        mg.add_block(block);
      }
      if let Some(lazy_info) = node.lazy_info {
        module_to_lazy_make.update_module_lazy_dependencies(mid, Some(lazy_info.into_owned()));
      }
      mg.add_module_graph_module(mgm);
      mg.add_module(module);
      continue;
    }
    let ids = node
      .dependencies
      .iter()
      .map(|(dep, _, _)| *dep.as_ref().id())
      .collect::<Vec<_>>();
    let input_dependencies = node
      .dependencies
      .iter()
      .map(|(dep, _, _)| (*dep.as_ref().id(), dep.as_ref()))
      .collect::<FxHashMap<_, _>>();
    if ids != mgm.all_dependencies()
      || ids.iter().any(|id| !dependency_ids.insert(*id))
      || !mgm.outgoing_connections().is_empty()
      || !matches!(mgm.issuer(), ModuleIssuer::Unset)
    {
      return Err(error!("inconsistent make cache build dependencies"));
    }
    let node_block_refs = node
      .blocks
      .iter()
      .map(|block| block.as_ref().clone())
      .collect::<Vec<_>>();
    if !valid_block_layout(module.as_ref())
      || !module
        .get_blocks()
        .iter()
        .copied()
        .eq(node_block_refs.iter().map(|block| block.identifier()))
    {
      return Err(error!("inconsistent make cache module blocks"));
    }
    let mut blocks = node
      .blocks
      .into_iter()
      .map(OwnedOrRef::into_owned)
      .collect::<Vec<_>>();
    let mut owned_blocks = FxHashMap::default();
    while let Some(block) = blocks.pop() {
      if block.parent() != &mid
        || !block_ids.insert(block.identifier())
        || !valid_block_layout(block.as_ref())
      {
        return Err(error!("inconsistent make cache async blocks"));
      }
      blocks.extend(block.get_block_refs().iter().cloned());
      owned_blocks.insert(block.identifier(), block);
    }
    // Retain the original per-block index, rather than the flattened all_dependencies index.
    // These inputs are also how we know that an injected/replaced dependency can be repaired.
    let mut build_inputs = FxHashMap::default();
    for (index, dep) in module.get_dependencies().iter().enumerate() {
      build_inputs.insert(*dep.id(), (None, index));
    }
    for (id, block) in &owned_blocks {
      for (index, dep) in block.get_dependencies().iter().enumerate() {
        build_inputs.insert(*dep.id(), (Some(*id), index));
      }
    }
    if build_inputs
      .keys()
      .any(|id| !input_dependencies.contains_key(id))
    {
      return Err(error!("missing make cache build dependency"));
    }
    for (dep, block, index) in &node.dependencies {
      if block
        .as_ref()
        .is_some_and(|block| !owned_blocks.contains_key(block.as_ref()))
      {
        return Err(error!("missing make cache dependency block"));
      }
      if let Some((expected_block, expected_index)) = build_inputs.get(dep.as_ref().id())
        && (expected_block.as_ref() != block.as_ref().map(OwnedOrRef::as_ref)
          || expected_index != index)
      {
        return Err(error!("inconsistent make cache dependency parents"));
      }
    }
    let mut expired = issuer.is_none();
    mgm.set_issuer(issuer.unwrap_or(ModuleIssuer::Unset));
    let mut refresh = FxHashSet::default();
    let mut resolved_dependencies = FxHashMap::default();
    let mut infos = vec![];
    let mut outgoing = vec![];
    let mut segmented = FxHashSet::default();
    for (ids, resolution) in resolutions {
      let group = ids.iter().copied().collect::<FxHashSet<_>>();
      if ids.is_empty()
        || group.len() != ids.len()
        || ids
          .iter()
          .any(|id| !input_dependencies.contains_key(id) || !segmented.insert(*id))
      {
        return Err(error!("inconsistent make cache resolution segment"));
      }
      let Some(resolution) = resolution else {
        // The group descriptor survives expiry; all related inputs are repaired together.
        expired = true;
        refresh.extend(ids);
        continue;
      };
      for dep in resolution
        .dependencies
        .into_iter()
        .map(OwnedOrRef::into_owned)
      {
        if !group.contains(dep.id()) || resolved_dependencies.insert(*dep.id(), dep).is_some() {
          return Err(error!("inconsistent make cache resolution dependencies"));
        }
      }
      if resolution.infos.len() > 1
        || (resolution.infos.is_empty() && !resolution.connections.is_empty())
      {
        return Err(error!("inconsistent make cache factorization group"));
      }
      for info in resolution.infos.into_iter().map(OwnedOrRef::into_owned) {
        let related = info
          .related_dep_ids()
          .iter()
          .copied()
          .collect::<FxHashSet<_>>();
        if related != group || related.len() != info.related_dep_ids().len() {
          return Err(error!("inconsistent make cache factorization group"));
        }
        // Materialized diagnostics must be regenerated from the reader's source view.
        if !info.is_success() {
          refresh.extend(info.related_dep_ids());
        }
        infos.push(info);
      }
      // Connection IDs are compilation-local. Partial cache saves can combine nodes from
      // different compilations whose IDs overlap; recovery allocates fresh IDs below.
      // Validate the stable owner/dependency relation instead of the serialized index.
      let mut connected = FxHashSet::default();
      for con in resolution
        .connections
        .into_iter()
        .map(OwnedOrRef::into_owned)
      {
        if con.original_module_identifier != Some(mid)
          || !group.contains(&con.dependency_id)
          || !connected.insert(con.dependency_id)
        {
          return Err(error!("inconsistent make cache connection"));
        }
        if !recovered.contains(con.module_identifier()) {
          refresh.extend(ids.iter().copied());
        }
        outgoing.push(con);
      }
    }
    if segmented.len() != input_dependencies.len() {
      return Err(error!("missing make cache resolution segment"));
    }
    let can_refactorize = module.as_normal_module().is_some()
      && node.lazy_info.as_ref().is_none_or(|lazy| {
        !lazy
          .as_ref()
          .all_lazy_dependencies()
          .any(|id| refresh.contains(&id))
      })
      && refresh.iter().all(|id| {
        input_dependencies.get(id).is_some_and(|dep| {
          // Context replacement mutates critical diagnostics in the dependency itself. Rebuild
          // those owners, and lazy/placeholder/injected inputs, rather than reuse resolved state.
          build_inputs.contains_key(id)
            && dep.lazy().is_none()
            && dep.as_module_dependency().is_some()
            && dep.as_context_dependency().is_none()
        })
      });
    let rebuild = (expired || !refresh.is_empty()) && !can_refactorize;
    check_issuer_cycles |= expired || !refresh.is_empty();
    for (dep, parent_block, index_in_block) in node.dependencies {
      let input = dep.into_owned();
      let resolved = resolved_dependencies.remove(input.id());
      let dep = if rebuild || refresh.contains(input.id()) {
        input
      } else {
        resolved.unwrap_or(input)
      };
      mg.set_parents(
        *dep.id(),
        DependencyParents {
          block: parent_block.map(OwnedOrRef::into_owned),
          module: mid,
          index_in_block,
        },
      );
      mg.add_dependency_ref(dep);
    }
    if rebuild {
      module = TempModule::transform_from(module.into(), node_block_refs).into_owned();
    } else {
      refactorize_dependencies.extend(refresh.iter().copied());
      for info in infos {
        if !info.related_dep_ids().iter().any(|id| refresh.contains(id)) {
          factorization_artifact.insert(info);
        }
      }
      for con in outgoing {
        if !refresh.contains(&con.dependency_id) {
          connections.push(con);
        }
      }
    }
    if mgm
      .issuer()
      .identifier()
      .is_some_and(|issuer| !recovered.contains(issuer))
    {
      mgm.set_issuer(ModuleIssuer::Unset);
    }
    for block in owned_blocks.into_values() {
      mg.add_block(block);
    }
    if let Some(lazy_info) = node.lazy_info {
      module_to_lazy_make.update_module_lazy_dependencies(mid, Some(lazy_info.into_owned()));
    }
    mg.add_module_graph_module(mgm);
    mg.add_module(module);
  }
  // recovery incoming connections
  for mut connection in connections {
    // Connection IDs belong to the recovered graph. Never allocate dense storage using an
    // untrusted serialized index, or retain IDs of edges discarded with an expired segment.
    connection.id = mg.next_connection_id();
    if let Some(origin) = connection.original_module_identifier {
      mg.module_graph_module_by_identifier_mut(&origin)
        .add_outgoing_connection(connection.id);
    }
    let mgm = mg.module_graph_module_by_identifier_mut(connection.module_identifier());
    mgm.add_incoming_connection(connection.id);
    mg.cache_recovery_connection(connection);
  }
  // recovery entry
  let mut entry_module: Vec<ModuleIdentifier> = vec![];
  for (_, mgm) in mg.module_graph_modules() {
    // Temporary entry edges keep disconnected targets alive until make repairs real entries
    // and refactorized edges. Cutting these anchors invokes the existing orphan/issuer repair.
    if check_issuer_cycles || mgm.issuer().identifier().is_none() {
      entry_module.push(mgm.module_identifier);
    };
  }
  let mut entry_dependencies: FxHashSet<DependencyId> = Default::default();
  for mid in entry_module {
    let dep = TempDependency::default();
    let connection =
      ModuleGraphConnection::new(mg.next_connection_id(), *dep.id(), None, mid, false);
    entry_dependencies.insert(*dep.id());
    mg.add_dependency(BoxDependency::new(dep));
    mg.cache_recovery_connection(connection);
  }

  tracing::debug!("recovery {} module", mg.modules_len());
  Ok(RecoveredModuleGraph {
    mg,
    factorization_artifact,
    module_to_lazy_make,
    entry_dependencies,
    refactorize_dependencies,
    check_issuer_cycles,
    dropped,
  })
}
