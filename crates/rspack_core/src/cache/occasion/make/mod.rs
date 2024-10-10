mod meta;
mod node;

use std::sync::atomic::Ordering::Relaxed;

use meta::{Meta, MetaRef};
use node::{Node, NodeRef};
use rayon::iter::*;
use rspack_cacheable::{from_bytes, to_bytes, DeserializeError};

use super::super::{cacheable::ArcCacheContext, ArcStorage};
use crate::{
  make::MakeArtifact, DependencyParents, ExportInfoData, ExportsInfoData, CONNECTION_ID,
  DEPENDENCY_ID,
};

#[derive(Debug)]
pub struct MakeOccasion {
  context: ArcCacheContext,
  storage: ArcStorage,
}

const MAKE_SCOPE: &'static str = "MAKE";
const META_SCOPE: &'static str = "MAKE_META";

impl MakeOccasion {
  pub fn new(storage: ArcStorage, context: ArcCacheContext) -> Self {
    Self { storage, context }
  }

  pub fn save(&self, artifact: &MakeArtifact) {
    let mg = artifact.get_module_graph();
    let total = artifact.built_modules.len();
    // save module_graph
    let nodes = artifact
      .built_modules
      .par_iter()
      .filter_map(|identifier| {
        let mgm = mg
          .module_graph_module_by_identifier(&identifier)
          .expect("should have mgm");
        let module = mg
          .module_by_identifier(identifier)
          .expect("should have module");
        let blocks = module
          .get_blocks()
          .par_iter()
          .map(|block_id| mg.block_by_id(block_id).expect("should have block").clone())
          .collect::<Vec<_>>();
        let dependencies = mgm
          .all_dependencies
          .par_iter()
          .map(|dep_id| {
            (
              mg.dependency_by_id(dep_id)
                .expect("should have dependency")
                .clone(),
              mg.get_parent_block(dep_id).cloned(),
            )
          })
          .collect::<Vec<_>>();
        let connections = mgm
          .outgoing_connections()
          .par_iter()
          .map(|con_id| {
            mg.connection_by_connection_id(con_id)
              .expect("should have connection")
              .clone()
          })
          .collect::<Vec<_>>();
        let node = NodeRef {
          mgm,
          module,
          dependencies,
          connections,
          blocks,
        };
        // TODO update context
        match to_bytes(&node, self.context.as_ref()) {
          Ok(bytes) => Some((identifier.as_bytes().to_vec(), bytes)),
          Err(_) => {
            // TODO add panic if all of struct are cacheable
            None
          }
        }
      })
      .collect::<Vec<_>>();

    // TODO updated deleted module

    let saved = nodes.len();
    println!("make save {}/{}", saved, total);

    for (id, bytes) in nodes {
      self.storage.set(MAKE_SCOPE, id, bytes)
    }
    // save meta
    let meta = MetaRef {
      make_failed_dependencies: &artifact.make_failed_dependencies,
      make_failed_module: &artifact.make_failed_module,
      file_dependencies: &artifact.file_dependencies,
      context_dependencies: &artifact.context_dependencies,
      missing_dependencies: &artifact.missing_dependencies,
      build_dependencies: &artifact.build_dependencies,
      next_dependencies_id: DEPENDENCY_ID.load(Relaxed),
      next_connection_id: CONNECTION_ID.load(Relaxed),
    };
    if let Ok(bytes) = to_bytes(&meta, self.context.as_ref()) {
      self
        .storage
        .set(META_SCOPE, "default".as_bytes().to_vec(), bytes);
    }
  }

  pub fn recovery(&self) -> Result<MakeArtifact, DeserializeError> {
    let mut artifact = MakeArtifact::default();
    for (k, v) in self.storage.get_all(META_SCOPE) {
      if String::from_utf8(k).unwrap() == "default" {
        let meta: Meta = from_bytes(&v, self.context.as_ref())?;
        artifact.make_failed_dependencies = meta.make_failed_dependencies;
        artifact.make_failed_module = meta.make_failed_module;
        // TODO save entry dependencies and connections
        artifact.entry_dependencies = Default::default();
        artifact.file_dependencies = meta.file_dependencies;
        artifact.context_dependencies = meta.context_dependencies;
        artifact.missing_dependencies = meta.missing_dependencies;
        artifact.build_dependencies = meta.build_dependencies;
        DEPENDENCY_ID.store(meta.next_dependencies_id, Relaxed);
        CONNECTION_ID.store(meta.next_connection_id, Relaxed);
      }
    }

    let mut need_check_dep = vec![];
    let mut mg = artifact.get_module_graph_mut();
    for (_, v) in self.storage.get_all(MAKE_SCOPE) {
      let mut node: Node = match from_bytes(&v, self.context.as_ref()) {
        Ok(n) => n,
        Err(_) => {
          continue;
        }
      };
      for (dep, parent_block) in node.dependencies {
        mg.set_parents(
          *dep.id(),
          DependencyParents {
            block: parent_block,
            module: node.module.identifier(),
          },
        );
        mg.add_dependency(dep);
      }
      for con in node.connections {
        need_check_dep.push((con.id, *con.module_identifier()));
        mg.cache_recovery_connection(con);
      }
      for block in node.blocks {
        mg.add_block(Box::new(block));
      }
      // recovery exports/export info
      let other_exports_info = ExportInfoData::new(None, None);
      let side_effects_only_info = ExportInfoData::new(Some("*side effects only*".into()), None);
      let exports_info = ExportsInfoData::new(other_exports_info.id(), side_effects_only_info.id());
      node.mgm.exports = exports_info.id();
      mg.set_exports_info(exports_info.id(), exports_info);
      mg.set_export_info(side_effects_only_info.id(), side_effects_only_info);
      mg.set_export_info(other_exports_info.id(), other_exports_info);

      mg.add_module_graph_module(node.mgm);
      mg.add_module(node.module);
    }
    // recovery incoming connections
    for (con_id, module_identifier) in &need_check_dep {
      if let Some(mgm) = mg.module_graph_module_by_identifier_mut(module_identifier) {
        mgm.add_incoming_connection(*con_id);
      }
    }

    println!("recovery {:?}", mg.modules().len());
    let build_deps = need_check_dep
      .iter()
      .filter_map(|(con_id, module_identifier)| {
        let module_exist = mg.module_by_identifier(&module_identifier).is_some();
        if module_exist {
          None
        } else {
          mg.revoke_connection(&con_id, false)
        }
      })
      .collect::<Vec<_>>();
    println!("recovery build deps size {:?}", build_deps.len());
    artifact.make_failed_dependencies.extend(build_deps);

    Ok(artifact)
  }
}
