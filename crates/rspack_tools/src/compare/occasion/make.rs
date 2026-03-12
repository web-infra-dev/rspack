use std::sync::Arc;

pub use rspack_core::cache::persistent::occasion::make::SCOPE;
use rspack_core::{
  DependencyId,
  build_module_graph::BuildModuleGraphArtifact,
  cache::persistent::{codec::CacheCodec, occasion::make::MakeOccasion, storage::Storage},
};
use rspack_error::Result;
use rspack_paths::Utf8PathBuf;
use rustc_hash::FxHashMap as HashMap;

use crate::{debug_info::DebugInfo, utils::ensure_iter_equal};

/// Compare make scope data between two storages
pub async fn compare(
  storage1: Arc<dyn Storage>,
  storage2: Arc<dyn Storage>,
  debug_info: DebugInfo,
) -> Result<()> {
  // Load make data from both storages
  let data1 = storage1.load(SCOPE).await?;
  let data2 = storage2.load(SCOPE).await?;

  // Convert to HashMap for easier comparison
  let map1: HashMap<_, _> = data1.into_iter().collect();
  let map2: HashMap<_, _> = data2.into_iter().collect();

  // Compare keys (module identifiers)
  ensure_iter_equal("Make module key", map1.keys(), map2.keys(), &debug_info)?;

  // Convert stored data to BuildModuleGraphArtifact using MakeOccasion's recovery ability
  // Use a dummy path for codec since we're only deserializing
  let codec = Arc::new(CacheCodec::new(Some(Utf8PathBuf::from("/"))));
  let occasion1 = MakeOccasion::new(storage1.clone(), codec.clone());
  let occasion2 = MakeOccasion::new(storage2.clone(), codec.clone());

  let artifact1 = occasion1.recovery().await?;
  let artifact2 = occasion2.recovery().await?;

  let comparator = ArtifactComparator::new(&artifact1, &artifact2);
  comparator.compare(&debug_info)?;

  Ok(())
}

/// Comparator for BuildModuleGraphArtifacts
struct ArtifactComparator<'a> {
  mg1: &'a rspack_core::ModuleGraph,
  mg2: &'a rspack_core::ModuleGraph,
}

impl<'a> ArtifactComparator<'a> {
  fn new(artifact1: &'a BuildModuleGraphArtifact, artifact2: &'a BuildModuleGraphArtifact) -> Self {
    let mg1 = &artifact1.module_graph;
    let mg2 = &artifact2.module_graph;
    Self { mg1, mg2 }
  }

  /// Compare the complete artifacts
  fn compare(&self, debug_info: &DebugInfo) -> Result<()> {
    // Get all modules from both graphs
    let modules1 = self
      .mg1
      .modules()
      .map(|(id, module)| (*id, module))
      .collect::<HashMap<_, _>>();
    let modules2 = self
      .mg2
      .modules()
      .map(|(id, module)| (*id, module))
      .collect::<HashMap<_, _>>();

    // Compare module keys
    ensure_iter_equal(
      "Module identifiers",
      modules1.keys(),
      modules2.keys(),
      debug_info,
    )?;

    // First pass: Compare dependencies and build DependencyId mapping
    // DependencyId mapping: dep_id1 -> dep_id2
    let mut dep_id_map: HashMap<DependencyId, DependencyId> = HashMap::default();

    for (module_id, module1) in &modules1 {
      let module2 = modules2
        .get(module_id)
        .expect("module should exist in both graphs");
      let module_debug_info = debug_info.with_field("module", &format!("{module_id:?}"));

      // Compare dependencies and build mapping
      self.compare_module_dependencies_and_build_map(
        module1,
        module2,
        &module_debug_info,
        &mut dep_id_map,
      )?;

      // Compare module relations (and verify dep_id_map)
      self.compare_module_relations(module_id, &module_debug_info, &dep_id_map)?;
    }

    // Second pass: Compare BuildInfo using the DependencyId mapping
    for (module_id, module1) in modules1 {
      let module2 = modules2
        .get(&module_id)
        .expect("module should exist in both graphs");
      let module_debug_info = debug_info.with_field("module", &format!("{module_id:?}"));

      self.compare_module_build_info(module1, module2, &module_debug_info, &dep_id_map)?;
    }

    Ok(())
  }

  /// Compare module's outgoing connections (downstream modules) and verify dep_id_map
  fn compare_module_relations(
    &self,
    module_id: &rspack_core::ModuleIdentifier,
    debug_info: &DebugInfo,
    dep_id_map: &HashMap<DependencyId, DependencyId>,
  ) -> Result<()> {
    // Get outgoing connections for this module from both graphs
    let connections1: Vec<_> = self.mg1.get_outgoing_connections(module_id).collect();
    let connections2: Vec<_> = self.mg2.get_outgoing_connections(module_id).collect();

    // Extract downstream module identifiers
    let downstream1: Vec<_> = connections1
      .iter()
      .map(|conn| conn.module_identifier())
      .collect();
    let downstream2: Vec<_> = connections2
      .iter()
      .map(|conn| conn.module_identifier())
      .collect();

    // Compare downstream module identifiers
    ensure_iter_equal(
      "Downstream module identifiers",
      downstream1.iter(),
      downstream2.iter(),
      debug_info,
    )?;

    // Verify dep_id_map: check that connection dependency IDs are correctly mapped
    for (conn1, conn2) in connections1.iter().zip(connections2.iter()) {
      let dep_id1 = conn1.dependency_id;
      let dep_id2 = conn2.dependency_id;

      let expected_dep_id2 = dep_id_map.get(&dep_id1).ok_or_else(|| {
        rspack_error::error!(
          "Connection DependencyId {:?} not found in dep_id_map\n{}",
          dep_id1,
          debug_info
        )
      })?;

      if expected_dep_id2 != &dep_id2 {
        return Err(rspack_error::error!(
          "Connection DependencyId mismatch: dep_id_map[{:?}] = {:?}, but connection has {:?}\n{}",
          dep_id1,
          expected_dep_id2,
          dep_id2,
          debug_info
        ));
      }
    }

    Ok(())
  }

  /// Compare module's dependencies and build DependencyId mapping
  /// DependencyId is not reliable across different builds, but the order from get_dependencies() is reliable.
  /// So we compare dependency_type in order and build a mapping from dep_id1 to dep_id2.
  fn compare_module_dependencies_and_build_map(
    &self,
    module1: &rspack_core::BoxModule,
    module2: &rspack_core::BoxModule,
    debug_info: &DebugInfo,
    dep_id_map: &mut HashMap<DependencyId, DependencyId>,
  ) -> Result<()> {
    let deps1 = module1.get_dependencies();
    let deps2 = module2.get_dependencies();

    // Compare dependency count
    if deps1.len() != deps2.len() {
      return Err(rspack_error::error!(
        "Module has different number of dependencies: {} vs {}\n{}",
        deps1.len(),
        deps2.len(),
        debug_info
      ));
    }

    // Compare each dependency by type in order and build mapping
    for (i, (dep_id1, dep_id2)) in deps1.iter().zip(deps2.iter()).enumerate() {
      let dep_debug_info = debug_info.with_field("dependency_index", &i.to_string());

      let dep1 = self.mg1.dependency_by_id(dep_id1);
      let dep2 = self.mg2.dependency_by_id(dep_id2);

      // Compare dependency types
      let type1 = dep1.dependency_type();
      let type2 = dep2.dependency_type();

      if type1 != type2 {
        return Err(rspack_error::error!(
          "Dependency type mismatch at index {}: {:?} vs {:?}\n{}",
          i,
          type1,
          type2,
          dep_debug_info
        ));
      }

      // Build mapping: dep_id1 -> dep_id2
      dep_id_map.insert(*dep_id1, *dep_id2);
    }

    Ok(())
  }

  /// Compare module's BuildInfo using DependencyId mapping
  /// We take an extreme approach: clone the BuildInfo, extract all_star_exports connections,
  /// and serialize the rest for direct comparison.
  fn compare_module_build_info(
    &self,
    module1: &rspack_core::BoxModule,
    module2: &rspack_core::BoxModule,
    debug_info: &DebugInfo,
    dep_id_map: &HashMap<DependencyId, DependencyId>,
  ) -> Result<()> {
    let build_info1 = module1.build_info();
    let build_info2 = module2.build_info();

    // Compare all_star_exports using the DependencyId mapping
    self.compare_all_star_exports(
      &build_info1.all_star_exports,
      &build_info2.all_star_exports,
      dep_id_map,
      debug_info,
    )?;

    // Clone BuildInfo and clear all_star_exports for serialization comparison
    let mut normalized_info1 = build_info1.clone();
    let mut normalized_info2 = build_info2.clone();
    normalized_info1.all_star_exports.clear();
    normalized_info2.all_star_exports.clear();

    // Serialize and compare the rest of BuildInfo using rspack_cacheable
    let ctx = ();
    let bytes1 = rspack_cacheable::to_bytes(&normalized_info1, &ctx).map_err(|e| {
      rspack_error::error!("Failed to serialize BuildInfo 1: {:?}\n{}", e, debug_info)
    })?;
    let bytes2 = rspack_cacheable::to_bytes(&normalized_info2, &ctx).map_err(|e| {
      rspack_error::error!("Failed to serialize BuildInfo 2: {:?}\n{}", e, debug_info)
    })?;

    if bytes1 != bytes2 {
      return Err(rspack_error::error!(
        "BuildInfo mismatch (excluding all_star_exports): serialized bytes differ\n{}",
        debug_info
      ));
    }

    Ok(())
  }

  /// Compare all_star_exports using DependencyId mapping
  fn compare_all_star_exports(
    &self,
    exports1: &[DependencyId],
    exports2: &[DependencyId],
    dep_id_map: &HashMap<DependencyId, DependencyId>,
    debug_info: &DebugInfo,
  ) -> Result<()> {
    if exports1.len() != exports2.len() {
      return Err(rspack_error::error!(
        "BuildInfo all_star_exports count mismatch: {} vs {}\n{}",
        exports1.len(),
        exports2.len(),
        debug_info
      ));
    }

    // Compare using DependencyId mapping
    for (i, dep_id1) in exports1.iter().enumerate() {
      let expected_dep_id2 = dep_id_map.get(dep_id1).ok_or_else(|| {
        rspack_error::error!(
          "BuildInfo all_star_exports[{}]: DependencyId {:?} not found in mapping\n{}",
          i,
          dep_id1,
          debug_info
        )
      })?;

      let actual_dep_id2 = &exports2[i];

      if expected_dep_id2 != actual_dep_id2 {
        return Err(rspack_error::error!(
          "BuildInfo all_star_exports[{}] mismatch: expected {:?}, got {:?}\n{}",
          i,
          expected_dep_id2,
          actual_dep_id2,
          debug_info
        ));
      }
    }

    Ok(())
  }
}
