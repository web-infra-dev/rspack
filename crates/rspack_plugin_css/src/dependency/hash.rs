use rspack_collections::IdentifierSet;
use rspack_core::{
  ChunkGraph, Compilation, DependencyCodeGeneration, DependencyId, GeneratorOptions, Module,
  RuntimeSpec,
};
use rspack_hash::{HashSalt, RspackHasher, rspack_hash_object};

use super::{
  CssIcssExportDependency, CssIcssImportDependency, CssImportDependency, CssUrlDependency,
};
use crate::utils::css_module_export_type;

/// Hash the binding, not the allocation-dependent dependency ID. Two uses of
/// the same name can refer to different definitions after a redefinition.
pub(super) fn hash_binding(compilation: &Compilation, id: DependencyId, hasher: &mut RspackHasher) {
  let graph = compilation.get_module_graph();
  let dependency = graph.dependency_by_id(&id);
  if let Some(export) = dependency.downcast_ref::<CssIcssExportDependency>() {
    rspack_hash_object!(hasher, {
      "type" => "export",
      "name" => &export.name,
      "value" => &export.value,
      "localIdent" => export.local_ident.map(|kind| kind as u8),
    });
  } else if let Some(import) = dependency.downcast_ref::<CssIcssImportDependency>() {
    let module_id = graph.get_module_by_dependency_id(&id).and_then(|target| {
      ChunkGraph::get_module_id(&compilation.module_ids_artifact, target.identifier())
    });
    rspack_hash_object!(hasher, {
      "type" => "import",
      "importName" => import.import_name(),
      "moduleId" => module_id,
    });
  }
}

pub(crate) fn hash_generator_options(module: &dyn Module, hasher: &mut RspackHasher) {
  let options = module
    .as_normal_module()
    .and_then(|module| module.get_generator_options());
  match options {
    Some(GeneratorOptions::CssModule(options)) => {
      let convention = options.exports_convention.unwrap_or_default();
      // Keep an absent salt distinct from a configured empty string.
      let salt = match &options.local_ident_hash_salt {
        HashSalt::None => (false, ""),
        HashSalt::Salt(salt) => (true, salt.as_str()),
      };
      rspack_hash_object!(hasher, {
        "exportsConvention" => [convention.as_is(), convention.camel_case(), convention.dashes()],
        "localIdentName" => options.local_ident_name.as_ref().map(|name| &name.template),
        "localIdentHashFunction" => options.local_ident_hash_function.map(|value| value as u8),
        "localIdentHashDigest" => options.local_ident_hash_digest.map(|value| value as u8),
        "localIdentHashDigestLength" => options.local_ident_hash_digest_length,
        "localIdentHashSalt" => salt,
        "exportsOnly" => options.exports_only,
        "esModule" => options.es_module,
      });
    }
    Some(GeneratorOptions::Css(options)) => {
      rspack_hash_object!(hasher, {
        "exportsOnly" => options.exports_only,
        "esModule" => options.es_module,
      });
    }
    _ => {}
  }
  rspack_hash_object!(hasher, {
    "exportType" => css_module_export_type(module),
  });
}

/// CSS values and generated identifiers are copied into the importing module.
/// Fingerprint their current build inputs, including transitive imports. Module
/// runtime hashes are computed in parallel and cannot be read here: the stored
/// hash can still belong to the previous compilation. Walk each target once,
/// without rendering CSS or recursively invoking module hash generation.
pub(super) fn hash_css_import_target(
  id: DependencyId,
  hasher: &mut RspackHasher,
  compilation: &Compilation,
  runtime: Option<&RuntimeSpec>,
) {
  let graph = compilation.get_module_graph();
  let mut visited = IdentifierSet::default();
  let mut visited_sources = IdentifierSet::default();
  if let Some(parent) = graph.get_parent_module(&id) {
    visited.insert(*parent);
    visited_sources.insert(*parent);
  }
  let include_import_sources = graph
    .dependency_by_id(&id)
    .downcast_ref::<CssImportDependency>()
    .is_some();
  let mut pending = vec![(id, include_import_sources)];
  while let Some((id, include_import_sources)) = pending.pop() {
    let Some(module) = graph.get_module_by_dependency_id(&id) else {
      continue;
    };
    // JS imports remain runtime property accesses; their source is not inlined.
    if module.build_info().css.is_none() {
      continue;
    }
    let first_visit = visited.insert(module.identifier());
    let visit_sources = include_import_sources && visited_sources.insert(module.identifier());
    if !first_visit && !visit_sources {
      continue;
    }
    if first_visit {
      rspack_hash_object!(hasher, {
        "buildHash" => &module.build_info().hash,
        "graphHash" => compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_graph_hash(module.as_ref(), compilation, runtime),
      });
      hash_generator_options(module.as_ref(), hasher);
      if let Some(dependencies) = module.get_presentational_dependencies() {
        for dependency in dependencies {
          dependency.update_hash(hasher, compilation, runtime);
        }
      }
    }
    // The build hash already covers definitions and source occurrences. Only
    // dependencies reading another module's build data need to be followed.
    // Reverse the stack additions to visit imports in source order.
    for dependency in module.get_dependencies().iter().rev() {
      if dependency
        .downcast_ref::<CssIcssImportDependency>()
        .is_some()
      {
        if first_visit {
          pending.push((*dependency.id(), false));
        }
      } else if let Some(import) = dependency.downcast_ref::<CssImportDependency>() {
        // An embedded stylesheet also embeds its own @imports, even if that
        // child's standalone generator would emit a separate CSS chunk.
        if visit_sources || (first_visit && import.is_inlined(compilation)) {
          pending.push((*dependency.id(), true));
        }
      } else if first_visit && let Some(url) = dependency.downcast_ref::<CssUrlDependency>() {
        url.update_hash(hasher, compilation, runtime);
      }
    }
  }
}
