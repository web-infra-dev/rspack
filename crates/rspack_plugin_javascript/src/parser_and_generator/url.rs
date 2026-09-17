use concat_string::concat_string;
use rspack_core::{
  AsyncDependenciesBlock, DependencyRef, EntryOptions, GroupOptions, ModuleDependency,
  ModuleFactoryCreateData, ModuleType, ParseContext, ParseResult,
};
use rspack_hash::{HashDigest, RspackHash, RspackHasher};
use rspack_util::identifier::split_at_query_mark;

use crate::dependency::{URLDependency, is_url_value_module};

pub(super) async fn promote_url_dependencies(
  result: &mut ParseResult,
  context: &mut ParseContext<'_>,
) {
  if !result
    .dependencies
    .iter()
    .any(|dep| dep.is::<URLDependency>())
  {
    return;
  }
  let dependencies = std::mem::take(&mut result.dependencies);
  result.dependencies.reserve(dependencies.len());
  for dependency in dependencies {
    let Some(url_dependency) = dependency.downcast_ref::<URLDependency>() else {
      result.dependencies.push(dependency);
      continue;
    };
    let build_context = context.build_context;
    let factory = build_context
      .dependency_factories
      .get(dependency.dependency_type())
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
    let promote = factory_result
      .ok()
      .and_then(|result| result.module)
      .is_some_and(|module| {
        !is_url_value_module(module.as_ref())
          && (module.module_type().is_js_like()
            || matches!(
              module.module_type(),
              ModuleType::Css | ModuleType::CssAuto | ModuleType::CssModule
            ))
      });
    if !promote {
      result.dependencies.push(dependency);
      continue;
    }

    let request = url_dependency.request().to_owned();
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
    result.blocks.push(block);
  }
}
