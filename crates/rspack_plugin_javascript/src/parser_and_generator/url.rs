use concat_string::concat_string;
use rspack_core::{
  AsyncDependenciesBlock, BoxDependency, DependenciesBlock, Dependency, DependencyId,
  DependencyRef, EntryOptions, GroupOptions, ModuleFactoryCreateData, ParseContext, ParseResult,
};
use rspack_hash::{HashDigest, RspackHash, RspackHasher};
use rspack_util::identifier::split_at_query_mark;
use rustc_hash::FxHashSet as HashSet;

use crate::dependency::{URLDependency, is_url_value_module};

pub(crate) fn iter_url_dependencies(result: &ParseResult) -> impl Iterator<Item = &URLDependency> {
  let mut blocks = result.blocks.iter().map(Box::as_ref).collect::<Vec<_>>();
  let block_dependencies = std::iter::from_fn(move || {
    let block = blocks.pop()?;
    blocks.extend(block.get_block_refs().iter().map(AsRef::as_ref));
    Some(block)
  })
  .flat_map(|block| block.get_dependencies())
  .filter_map(|dependency| dependency.downcast_ref::<URLDependency>());

  block_dependencies.chain(
    result
      .dependencies
      .iter()
      .filter_map(|dependency| dependency.downcast_ref::<URLDependency>()),
  )
}

pub(crate) fn apply_url_dependency_promotions(
  result: &mut ParseResult,
  context: &ParseContext<'_>,
  promoted_dependencies: &HashSet<DependencyId>,
) {
  let mut blocks = result
    .blocks
    .iter_mut()
    .map(Box::as_mut)
    .collect::<Vec<_>>();
  while let Some(block) = blocks.pop() {
    let existing_blocks = block.get_blocks().len();
    let mut entries = Vec::new();
    for dependency in block.get_dependencies() {
      if promoted_dependencies.contains(dependency.id())
        && let Some(url_dependency) = dependency.downcast_ref::<URLDependency>()
      {
        entries.push((
          *dependency.id(),
          create_url_entry(BoxDependency::new(url_dependency.clone()), context),
        ));
      }
    }
    for (dependency_id, entry) in entries {
      block.remove_dependency_id(dependency_id);
      block.add_block(entry.into());
    }
    // Only visit parser-created blocks; new URL entries already contain promoted dependencies.
    blocks.extend(block.blocks_mut().take(existing_blocks));
  }

  if !result
    .dependencies
    .iter()
    .any(|dep| promoted_dependencies.contains(dep.id()))
  {
    return;
  }
  let dependencies = std::mem::take(&mut result.dependencies);
  result.dependencies.reserve(dependencies.len());
  for dependency in dependencies {
    if promoted_dependencies.contains(dependency.id()) {
      result.blocks.push(create_url_entry(dependency, context));
    } else {
      result.dependencies.push(dependency);
    }
  }
}

pub(crate) async fn should_promote_url_dependency(
  url_dependency: &URLDependency,
  context: &mut ParseContext<'_>,
) -> bool {
  let build_context = context.build_context;
  let factory = build_context
    .dependency_factories
    .get(url_dependency.dependency_type())
    .expect("URL dependency should have a module factory");
  let issuer_resource = context
    .module_match_resource
    .unwrap_or(context.resource_data);
  let mut create_data = ModuleFactoryCreateData::new(
    build_context.clone(),
    context.module_resolve_options.clone(),
    Some(context.module_context),
    vec![DependencyRef::new(url_dependency.clone())],
    Some(split_at_query_mark(issuer_resource.resource()).0.into()),
    Some(context.module_identifier),
    context.module_layer.cloned(),
  );
  // The probe does not build or retain the target module. Its resolution inputs
  // belong to the issuer too, since its dependency layout depends on the result.
  let factory_result = factory.create(&mut create_data).await;
  context
    .build_info
    .dependencies
    .file
    .extend(create_data.file_dependencies);
  context
    .build_info
    .dependencies
    .context
    .extend(create_data.context_dependencies);
  context
    .build_info
    .dependencies
    .missing
    .extend(create_data.missing_dependencies);
  // Let normal factorization report failures with its usual diagnostics and
  // bail behavior, instead of turning a failed probe into an issuer build error.
  factory_result
    .ok()
    .and_then(|result| result.module)
    .is_some_and(|module| {
      !is_url_value_module(module.as_ref()) && module.module_type().is_js_like()
    })
}

fn create_url_entry(
  dependency: BoxDependency,
  context: &ParseContext<'_>,
) -> Box<AsyncDependenciesBlock> {
  let request = dependency
    .as_module_dependency()
    .expect("URL dependency")
    .request()
    .to_owned();
  let mut hasher = RspackHasher::from(&context.compiler_options.output);
  context.module_identifier.hash(&mut hasher);
  request.hash(&mut hasher);
  dependency.range().hash(&mut hasher);
  let hash = hasher.digest(&HashDigest::Hex);
  let runtime = concat_string!("url-", hash.rendered(16));
  let mut block = Box::new(AsyncDependenciesBlock::new(
    context.module_identifier,
    dependency.loc(),
    None,
    vec![dependency],
    Some(request),
  ));
  block.set_group_options(GroupOptions::Entrypoint(Box::new(EntryOptions {
    runtime: Some(runtime.into()),
    ..Default::default()
  })));
  block
}
