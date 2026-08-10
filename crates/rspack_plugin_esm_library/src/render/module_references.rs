use rspack_collections::IdentifierIndexMap;
use rspack_core::{
  CodeGenerationModuleReferenceKind, CodeGenerationModuleReferences, Compilation, ModuleInfo,
  rspack_sources::{BoxSource, ReplaceSource, Source, SourceExt},
};
use rspack_error::{Result, error};
use rspack_util::fx_hash::FxHashMap;

use crate::{EsmLibraryPlugin, chunk_link::ChunkLinkContext, is_css_only_module};

fn local_namespace<'a>(
  module: rspack_core::ModuleIdentifier,
  chunk_link: &'a ChunkLinkContext,
  fallback: Option<&'a rspack_util::atom::Atom>,
  esm_namespace: bool,
) -> Result<&'a rspack_util::atom::Atom> {
  let namespace = if esm_namespace {
    chunk_link.esm_namespace_objects.get(&module)
  } else {
    None
  };
  namespace
    .or_else(|| chunk_link.hoisted_namespaces.get(&module))
    .or(fallback)
    .ok_or_else(|| error!("missing namespace for scope-hoisted module {module}"))
}

fn local_module_value_with_namespace(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
  esm_namespace: bool,
) -> Result<String> {
  if let Some(target) = compilation.get_module_graph().module_by_identifier(&module)
    && is_css_only_module(target.as_ref(), compilation.get_module_graph())
  {
    return Ok("undefined".to_string());
  }
  let info = module_infos
    .get(&module)
    .ok_or_else(|| error!("missing modern-module link info for {module}"))?;
  match info {
    ModuleInfo::Wrapped(_) => chunk_link
      .module_initializers
      .get(&module)
      .map(|initializer| format!("{initializer}()"))
      .ok_or_else(|| error!("missing initializer for wrapped module {module}")),
    ModuleInfo::Concatenated(info) => {
      let namespace = local_namespace(
        module,
        chunk_link,
        info.namespace_object_name.as_ref(),
        esm_namespace,
      )?;
      Ok(chunk_link.module_initializers.get(&module).map_or_else(
        || namespace.to_string(),
        |initializer| format!("({initializer}(), {namespace})"),
      ))
    }
    ModuleInfo::External(_) => Err(error!(
      "external module info is not used by modern-module rendering"
    )),
  }
}

fn local_module_value(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  local_module_value_with_namespace(module, compilation, chunk_link, module_infos, false)
}

fn local_esm_module_value(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  local_module_value_with_namespace(module, compilation, chunk_link, module_infos, true)
}

fn imported_module_value(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
) -> Result<String> {
  let target_chunk = EsmLibraryPlugin::get_module_chunk(module, compilation)?;
  let target_link = links
    .get(&target_chunk)
    .ok_or_else(|| error!("missing modern-module chunk link for {target_chunk:?}"))?;
  let initializer = target_link
    .module_initializer_exports
    .get(&module)
    .ok_or_else(|| error!("missing exported initializer for module {module}"))?;
  let is_async = rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, &module);
  if target_link.hoisted_initializers.contains(&module) {
    let namespace = target_link
      .initializer_namespace_exports
      .get(&module)
      .map_or_else(|| "m".to_string(), |namespace| format!("m.{namespace}"));
    if is_async {
      Ok(format!(
        "m => Promise.resolve(m.{initializer}()).then(() => {namespace})"
      ))
    } else {
      Ok(format!("m => (m.{initializer}(), {namespace})"))
    }
  } else if is_async {
    Ok(format!("m => Promise.resolve(m.{initializer}())"))
  } else {
    Ok(format!("m => m.{initializer}()"))
  }
}

fn local_async_module_value(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  if let Some(target) = compilation.get_module_graph().module_by_identifier(&module)
    && is_css_only_module(target.as_ref(), compilation.get_module_graph())
  {
    return Ok("Promise.resolve(undefined)".to_string());
  }
  let info = module_infos
    .get(&module)
    .ok_or_else(|| error!("missing modern-module link info for {module}"))?;
  match info {
    ModuleInfo::Wrapped(_) => {
      let initializer = chunk_link
        .module_initializers
        .get(&module)
        .ok_or_else(|| error!("missing initializer for wrapped module {module}"))?;
      Ok(format!("Promise.resolve().then(() => {initializer}())"))
    }
    ModuleInfo::Concatenated(info) => {
      let namespace = local_namespace(
        module,
        chunk_link,
        info.namespace_object_name.as_ref(),
        true,
      )?;
      Ok(chunk_link.module_initializers.get(&module).map_or_else(
        || format!("Promise.resolve({namespace})"),
        |initializer| {
          format!("Promise.resolve().then(() => {initializer}()).then(() => {namespace})")
        },
      ))
    }
    ModuleInfo::External(_) => Err(error!(
      "external module info is not used by modern-module rendering"
    )),
  }
}

fn local_module_initializer(
  module: rspack_core::ModuleIdentifier,
  chunk_link: &ChunkLinkContext,
) -> Result<String> {
  chunk_link
    .module_initializers
    .get(&module)
    .map(|initializer| format!("() => {initializer}()"))
    .ok_or_else(|| error!("missing initializer for module {module}"))
}

fn imported_module_loader(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
) -> Result<String> {
  let target_chunk = EsmLibraryPlugin::get_module_chunk(module, compilation)?;
  let target_link = links
    .get(&target_chunk)
    .ok_or_else(|| error!("missing modern-module chunk link for {target_chunk:?}"))?;
  let initializer = target_link
    .module_initializer_exports
    .get(&module)
    .ok_or_else(|| error!("missing exported initializer for module {module}"))?;
  if target_link.hoisted_initializers.contains(&module) {
    Ok(format!(
      "m => () => (m.{initializer}(), {})",
      target_link
        .initializer_namespace_exports
        .get(&module)
        .map_or_else(|| "m".to_string(), |namespace| format!("m.{namespace}"))
    ))
  } else {
    Ok(format!("m => () => m.{initializer}()"))
  }
}

fn async_module_value(
  module: rspack_core::ModuleIdentifier,
  current_chunk: rspack_core::ChunkUkey,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  let target_chunk = EsmLibraryPlugin::get_module_chunk(module, compilation)?;
  if target_chunk == current_chunk {
    if rspack_core::ModuleGraph::is_async(&compilation.async_modules_artifact, &module) {
      return local_async_module_value(module, compilation, chunk_link, module_infos);
    }
    return Ok(format!(
      "Promise.resolve().then(() => {})",
      local_esm_module_value(module, compilation, chunk_link, module_infos)?
    ));
  }
  let target = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&target_chunk);
  Ok(format!(
    "import(\"__RSPACK_ESM_CHUNK_{}\").then({})",
    target.expect_id().as_str(),
    imported_module_value(module, compilation, links)?
  ))
}

fn async_module_initializer(
  module: rspack_core::ModuleIdentifier,
  current_chunk: rspack_core::ChunkUkey,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
) -> Result<String> {
  let target_chunk = EsmLibraryPlugin::get_module_chunk(module, compilation)?;
  if target_chunk == current_chunk {
    let initializer = chunk_link
      .module_initializers
      .get(&module)
      .ok_or_else(|| error!("missing initializer for module {module}"))?;
    return Ok(format!("Promise.resolve().then(() => {initializer}())"));
  }
  let target = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&target_chunk);
  let target_link = links
    .get(&target_chunk)
    .ok_or_else(|| error!("missing modern-module chunk link for {target_chunk:?}"))?;
  let initializer = target_link
    .module_initializer_exports
    .get(&module)
    .ok_or_else(|| error!("missing exported initializer for module {module}"))?;
  Ok(format!(
    "import(\"__RSPACK_ESM_CHUNK_{}\").then(m => m.{initializer}())",
    target.expect_id().as_str()
  ))
}

fn async_module_loader(
  module: rspack_core::ModuleIdentifier,
  current_chunk: rspack_core::ChunkUkey,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  let target_chunk = EsmLibraryPlugin::get_module_chunk(module, compilation)?;
  if target_chunk == current_chunk {
    return Ok(format!(
      "Promise.resolve(() => {})",
      local_esm_module_value(module, compilation, chunk_link, module_infos)?
    ));
  }
  let target = compilation
    .build_chunk_graph_artifact
    .chunk_by_ukey
    .expect_get(&target_chunk);
  Ok(format!(
    "import(\"__RSPACK_ESM_CHUNK_{}\").then({})",
    target.expect_id().as_str(),
    imported_module_loader(module, compilation, links)?
  ))
}

fn weak_module_value(
  module: rspack_core::ModuleIdentifier,
  compilation: &Compilation,
  chunk_link: &ChunkLinkContext,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<String> {
  if !module_infos.contains_key(&module) {
    return Ok("undefined".to_string());
  }
  Ok(format!(
    "() => {}",
    local_module_value(module, compilation, chunk_link, module_infos)?
  ))
}

pub(super) fn relocate_module_references(
  source: BoxSource,
  references: Option<&CodeGenerationModuleReferences>,
  compilation: &Compilation,
  current_chunk: rspack_core::ChunkUkey,
  chunk_link: &ChunkLinkContext,
  links: &FxHashMap<rspack_core::ChunkUkey, ChunkLinkContext>,
  module_infos: &IdentifierIndexMap<ModuleInfo>,
) -> Result<BoxSource> {
  let Some(references) = references else {
    return Ok(source);
  };
  let content = source.source().into_string_lossy().into_owned();
  let mut relocated = ReplaceSource::new(source);

  for reference in references.iter() {
    let value = match reference.kind {
      CodeGenerationModuleReferenceKind::EntryValue => compilation
        .build_chunk_graph_artifact
        .chunk_graph
        .get_chunk_entry_modules(&current_chunk)
        .contains(&reference.module)
        .to_string(),
      CodeGenerationModuleReferenceKind::Value => {
        local_module_value(reference.module, compilation, chunk_link, module_infos)?
      }
      CodeGenerationModuleReferenceKind::ConstructorValue => format!(
        "new (function(value) {{ return value; }})({})",
        local_module_value(reference.module, compilation, chunk_link, module_infos)?
      ),
      CodeGenerationModuleReferenceKind::LazyValue => format!(
        "() => {}",
        local_esm_module_value(reference.module, compilation, chunk_link, module_infos)?
      ),
      CodeGenerationModuleReferenceKind::LazyInitializer => {
        local_module_initializer(reference.module, chunk_link)?
      }
      CodeGenerationModuleReferenceKind::ImportedValue => {
        imported_module_value(reference.module, compilation, links)?
      }
      CodeGenerationModuleReferenceKind::ImportedLazyValue => {
        imported_module_loader(reference.module, compilation, links)?
      }
      CodeGenerationModuleReferenceKind::AsyncValue => async_module_value(
        reference.module,
        current_chunk,
        compilation,
        chunk_link,
        links,
        module_infos,
      )?,
      CodeGenerationModuleReferenceKind::AsyncInitializer => async_module_initializer(
        reference.module,
        current_chunk,
        compilation,
        chunk_link,
        links,
      )?,
      CodeGenerationModuleReferenceKind::AsyncLazyValue => async_module_loader(
        reference.module,
        current_chunk,
        compilation,
        chunk_link,
        links,
        module_infos,
      )?,
      CodeGenerationModuleReferenceKind::WeakValue => {
        weak_module_value(reference.module, compilation, chunk_link, module_infos)?
      }
    };

    let mut found = false;
    for (start, _) in content.match_indices(&reference.marker) {
      found = true;
      let start: u32 = start
        .try_into()
        .expect("generated module source offset should fit in u32");
      let end = start
        + u32::try_from(reference.marker.len())
          .expect("generated module relocation length should fit in u32");
      relocated.replace(start, end, value.clone(), None);
    }
    if !found {
      return Err(error!(
        "module relocation marker {} was not found in generated source",
        reference.marker
      ));
    }
  }

  Ok(relocated.boxed())
}
