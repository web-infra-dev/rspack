use rspack_collections::IdentifierSet;
use rspack_core::{
  Compilation, DependencyCodeGeneration, DependencyId, GeneratorOptions, Module, ModuleGraph,
  RuntimeSpec,
};
use rspack_hash::{HashSalt, RspackHash, RspackHasher, write_u64_hex};

use super::{
  CssIcssExportDependency, CssIcssImportDependency, CssImportDependency, CssUrlDependency,
};
use crate::utils::css_module_export_type;

pub(super) fn hash_field(value: &str, hasher: &mut RspackHasher) {
  value.len().hash(hasher);
  hasher.write(b":");
  value.hash(hasher);
}

/// Hash the binding, not the allocation-dependent dependency ID. Two uses of
/// the same name can refer to different definitions after a redefinition.
pub(super) fn hash_binding(graph: &ModuleGraph, id: DependencyId, hasher: &mut RspackHasher) {
  let dependency = graph.dependency_by_id(&id);
  if let Some(export) = dependency.downcast_ref::<CssIcssExportDependency>() {
    hasher.write(b"export");
    hash_field(&export.name, hasher);
    hash_field(&export.value, hasher);
    export.local_ident.map(|kind| kind as u8).hash(hasher);
  } else if let Some(import) = dependency.downcast_ref::<CssIcssImportDependency>() {
    hasher.write(b"import");
    hash_field(import.import_name(), hasher);
    if let Some(target) = graph.get_module_by_dependency_id(&id) {
      hash_field(target.identifier().as_str(), hasher);
    }
  }
}

pub(crate) fn hash_generator_options(module: &dyn Module, hasher: &mut RspackHasher) {
  let options = module
    .as_normal_module()
    .and_then(|module| module.get_generator_options());
  match options {
    Some(GeneratorOptions::CssModule(options)) => {
      let convention = options.exports_convention.unwrap_or_default();
      [
        convention.as_is(),
        convention.camel_case(),
        convention.dashes(),
      ]
      .hash(hasher);
      options
        .local_ident_name
        .as_ref()
        .map(|name| &name.template)
        .hash(hasher);
      hasher.write(b"|function:");
      options
        .local_ident_hash_function
        .map(|value| value as u8)
        .hash(hasher);
      hasher.write(b"|digest:");
      options
        .local_ident_hash_digest
        .map(|value| value as u8)
        .hash(hasher);
      hasher.write(b"|length:");
      options.local_ident_hash_digest_length.hash(hasher);
      hasher.write(b"|salt:");
      match &options.local_ident_hash_salt {
        HashSalt::None => hasher.write(b"none"),
        HashSalt::Salt(salt) => {
          hasher.write(b"some");
          hash_field(salt, hasher);
        }
      }
      (options.exports_only, options.es_module).hash(hasher);
    }
    Some(GeneratorOptions::Css(options)) => {
      (options.exports_only, options.es_module).hash(hasher);
    }
    _ => {}
  }
  css_module_export_type(module).hash(hasher);
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
      hasher.write(b"|css-target:");
      hash_field(module.identifier().as_str(), hasher);
      module.build_info().hash.hash(hasher);
      hasher.write(b"|graph:");
      write_u64_hex(
        compilation
          .build_chunk_graph_artifact
          .chunk_graph
          .get_module_graph_hash(module.as_ref(), compilation, runtime),
        hasher,
      );
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
