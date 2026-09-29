use std::borrow::Cow;

use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_loader_runner::ResourceData;
use rspack_paths::Utf8Path;

use crate::{
  DataRef, DependencyCategory, ImportAttributes, ImportPhase, ModuleRule, ModuleRuleEffect,
  RuleSetConditionWithEmpty,
};

pub struct MatchContext<'ctx> {
  pub resource_data: &'ctx ResourceData,
  pub issuer: Option<&'ctx str>,
  pub issuer_layer: Option<&'ctx str>,
  pub dependency: DependencyCategory,
  pub phase: ImportPhase,
  pub attributes: Option<&'ctx ImportAttributes>,
}

pub async fn module_rules_matcher<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  ctx: &MatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<()> {
  let matched_rules_len = matched_rules.len();
  let mut resource_data = Cow::Borrowed(ctx.resource_data);
  if let Some(result) = module_rules_matcher_sync(rules, ctx, &mut resource_data, matched_rules) {
    return result;
  }
  matched_rules.truncate(matched_rules_len);
  resource_data = Cow::Borrowed(ctx.resource_data);
  module_rules_matcher_async(rules, ctx, &mut resource_data, matched_rules).await
}

fn module_rules_matcher_sync<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  ctx: &MatchContext<'ctx>,
  resource_data: &mut Cow<'ctx, ResourceData>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Option<Result<()>> {
  for rule in rules {
    let original_resource_data = rule.one_of.as_ref().map(|_| resource_data.clone());
    match module_rule_matcher_sync(rule, ctx, resource_data, matched_rules) {
      Some(Ok(true)) => {}
      Some(Ok(false)) => {
        if let Some(original_resource_data) = original_resource_data {
          *resource_data = original_resource_data;
        }
      }
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  }
  Some(Ok(()))
}

async fn module_rules_matcher_async<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  ctx: &MatchContext<'ctx>,
  resource_data: &mut Cow<'ctx, ResourceData>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<()> {
  for rule in rules {
    let original_resource_data = rule.one_of.as_ref().map(|_| resource_data.clone());
    if !module_rule_matcher_async(rule, ctx, resource_data, matched_rules).await? {
      if let Some(original_resource_data) = original_resource_data {
        *resource_data = original_resource_data;
      }
    }
  }
  Ok(())
}

fn apply_rule_as(pattern: &str, resource_data: &mut ResourceData) {
  let Some(path) = resource_data.path() else {
    return;
  };
  let filename = pattern.replace('*', path.file_stem().unwrap_or_default());
  let path = path.with_file_name(filename);
  let mut resource = path.as_str().to_owned();
  if let Some(query) = resource_data.query() {
    resource.push_str(query);
  }
  if let Some(fragment) = resource_data.fragment() {
    resource.push_str(fragment);
  }
  resource_data.set_path(path);
  resource_data.set_resource(resource);
}

macro_rules! ensure_sync_matched {
  ($result:expr) => {
    match $result {
      Some(Ok(true)) => {}
      Some(Ok(false)) => return Some(Ok(false)),
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  };
}

macro_rules! ensure_sync_not_matched {
  ($result:expr) => {
    match $result {
      Some(Ok(true)) => return Some(Ok(false)),
      Some(Ok(false)) => {}
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  };
}

fn check_optional_sync(
  condition: &RuleSetConditionWithEmpty,
  data: Option<DataRef<'_>>,
) -> Option<Result<bool>> {
  match data {
    Some(data) => condition.try_match_sync(data),
    None => condition.match_when_empty_sync(),
  }
}

async fn check_optional_async(
  condition: &RuleSetConditionWithEmpty,
  data: Option<DataRef<'_>>,
) -> Result<bool> {
  match data {
    Some(data) => condition.try_match(data).await,
    None => condition.match_when_empty().await,
  }
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
pub async fn module_rule_matcher<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  ctx: &MatchContext<'ctx>,
  resource_data: &mut Cow<'ctx, ResourceData>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  let matched_rules_len = matched_rules.len();
  let original_resource_data = resource_data.clone();
  if let Some(result) = module_rule_matcher_sync(module_rule, ctx, resource_data, matched_rules) {
    return result;
  }
  matched_rules.truncate(matched_rules_len);
  *resource_data = original_resource_data;
  module_rule_matcher_async(module_rule, ctx, resource_data, matched_rules).await
}

fn module_rule_matcher_sync<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  ctx: &MatchContext<'ctx>,
  resource_data: &mut Cow<'ctx, ResourceData>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Option<Result<bool>> {
  if let Some(test_rule) = &module_rule.rspack_resource {
    ensure_sync_matched!(test_rule.try_match_sync(resource_data.resource().into()));
  }

  let resource_path = resource_data
    .path()
    .unwrap_or_else(|| Utf8Path::new(""))
    .as_str();

  let test_unmatched = if let Some(test_rule) = &module_rule.test {
    match test_rule.try_match_sync(resource_path.into()) {
      Some(Ok(true)) => false,
      Some(Ok(false)) => true,
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  } else {
    false
  };
  if test_unmatched {
    return Some(Ok(false));
  } else if let Some(resource_rule) = &module_rule.resource {
    ensure_sync_matched!(resource_rule.try_match_sync(resource_path.into()));
  }

  if let Some(include_rule) = &module_rule.include {
    ensure_sync_matched!(include_rule.try_match_sync(resource_path.into()));
  }

  if let Some(exclude_rule) = &module_rule.exclude {
    ensure_sync_not_matched!(exclude_rule.try_match_sync(resource_path.into()));
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    ensure_sync_matched!(check_optional_sync(
      resource_query_rule,
      resource_data.query().map(Into::into),
    ));
  }

  if let Some(resource_fragment_condition) = &module_rule.resource_fragment {
    ensure_sync_matched!(check_optional_sync(
      resource_fragment_condition,
      resource_data.fragment().map(Into::into),
    ));
  }

  if let Some(mimetype_condition) = &module_rule.mimetype {
    ensure_sync_matched!(check_optional_sync(
      mimetype_condition,
      resource_data.mimetype().map(Into::into),
    ));
  }

  if let Some(scheme_condition) = &module_rule.scheme {
    let scheme = resource_data.get_scheme();
    if scheme.is_none() {
      ensure_sync_matched!(scheme_condition.match_when_empty_sync());
    }
    ensure_sync_matched!(scheme_condition.try_match_sync(scheme.as_str().into()));
  }

  if let Some(issuer_rule) = &module_rule.issuer {
    ensure_sync_matched!(check_optional_sync(issuer_rule, ctx.issuer.map(Into::into),));
  }

  if let Some(issuer_layer_rule) = &module_rule.issuer_layer {
    ensure_sync_matched!(check_optional_sync(
      issuer_layer_rule,
      ctx.issuer_layer.map(Into::into),
    ));
  }

  if let Some(dependency_rule) = &module_rule.dependency {
    ensure_sync_matched!(dependency_rule.try_match_sync(ctx.dependency.as_str().into()));
  }

  if let Some(phase_rule) = &module_rule.phase {
    ensure_sync_matched!(phase_rule.try_match_sync(ctx.phase.as_str().into()));
  }

  if let Some(description_data) = &module_rule.description_data {
    if let Some(resource_description) = resource_data.description() {
      for (k, matcher) in description_data {
        ensure_sync_matched!(check_optional_sync(
          matcher,
          k.split('.')
            .try_fold(resource_description.json(), |acc, key| acc.get(key))
            .map(Into::into),
        ));
      }
    } else {
      for matcher in description_data.values() {
        ensure_sync_matched!(matcher.match_when_empty_sync());
      }
    }
  }

  if let Some(with) = &module_rule.with {
    if let Some(attributes) = ctx.attributes {
      for (k, matcher) in with {
        ensure_sync_matched!(check_optional_sync(
          matcher,
          attributes.get(k).map(Into::into),
        ));
      }
    } else {
      for matcher in with.values() {
        ensure_sync_matched!(matcher.match_when_empty_sync());
      }
    }
  }

  matched_rules.push(&module_rule.effect);
  if let Some(pattern) = &module_rule.r#as {
    apply_rule_as(pattern, resource_data.to_mut());
  }

  if let Some(rules) = &module_rule.rules {
    match module_rules_matcher_sync(rules, ctx, resource_data, matched_rules) {
      Some(Ok(())) => {}
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      let original_resource_data = rule.one_of.as_ref().map(|_| resource_data.clone());
      match module_rule_matcher_sync(rule, ctx, resource_data, matched_rules) {
        Some(Ok(true)) => {
          matched_once = true;
          break;
        }
        Some(Ok(false)) => {
          if let Some(original_resource_data) = original_resource_data {
            *resource_data = original_resource_data;
          }
        }
        Some(Err(err)) => return Some(Err(err)),
        None => return None,
      }
    }
    if !matched_once {
      return Some(Ok(false));
    }
  }

  Some(Ok(true))
}

#[async_recursion]
async fn module_rule_matcher_async<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  ctx: &MatchContext<'ctx>,
  resource_data: &mut Cow<'ctx, ResourceData>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  if let Some(test_rule) = &module_rule.rspack_resource
    && !test_rule.try_match(resource_data.resource().into()).await?
  {
    return Ok(false);
  }

  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  let resource_path = resource_data
    .path()
    .unwrap_or_else(|| Utf8Path::new(""))
    .as_str();

  if let Some(test_rule) = &module_rule.test
    && !test_rule.try_match(resource_path.into()).await?
  {
    return Ok(false);
  } else if let Some(resource_rule) = &module_rule.resource
    && !resource_rule.try_match(resource_path.into()).await?
  {
    return Ok(false);
  }

  if let Some(include_rule) = &module_rule.include
    && !include_rule.try_match(resource_path.into()).await?
  {
    return Ok(false);
  }

  if let Some(exclude_rule) = &module_rule.exclude
    && exclude_rule.try_match(resource_path.into()).await?
  {
    return Ok(false);
  }

  if let Some(resource_query_rule) = &module_rule.resource_query
    && !check_optional_async(resource_query_rule, resource_data.query().map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(resource_fragment_condition) = &module_rule.resource_fragment
    && !check_optional_async(
      resource_fragment_condition,
      resource_data.fragment().map(Into::into),
    )
    .await?
  {
    return Ok(false);
  }

  if let Some(mimetype_condition) = &module_rule.mimetype
    && !check_optional_async(mimetype_condition, resource_data.mimetype().map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(scheme_condition) = &module_rule.scheme {
    let scheme = resource_data.get_scheme();
    if scheme.is_none() && !scheme_condition.match_when_empty().await? {
      return Ok(false);
    }
    if !scheme_condition.try_match(scheme.as_str().into()).await? {
      return Ok(false);
    }
  }

  if let Some(issuer_rule) = &module_rule.issuer
    && !check_optional_async(issuer_rule, ctx.issuer.map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(issuer_layer_rule) = &module_rule.issuer_layer
    && !check_optional_async(issuer_layer_rule, ctx.issuer_layer.map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(dependency_rule) = &module_rule.dependency
    && !dependency_rule
      .try_match(ctx.dependency.as_str().into())
      .await?
  {
    return Ok(false);
  }

  if let Some(phase_rule) = &module_rule.phase
    && !phase_rule.try_match(ctx.phase.as_str().into()).await?
  {
    return Ok(false);
  }

  if let Some(description_data) = &module_rule.description_data {
    if let Some(resource_description) = resource_data.description() {
      for (k, matcher) in description_data {
        if !check_optional_async(
          matcher,
          k.split('.')
            .try_fold(resource_description.json(), |acc, key| acc.get(key))
            .map(Into::into),
        )
        .await?
        {
          return Ok(false);
        }
      }
    } else {
      for matcher in description_data.values() {
        if !matcher.match_when_empty().await? {
          return Ok(false);
        }
      }
    }
  }

  if let Some(with) = &module_rule.with {
    if let Some(attributes) = ctx.attributes {
      for (k, matcher) in with {
        if !check_optional_async(matcher, attributes.get(k).map(Into::into)).await? {
          return Ok(false);
        }
      }
    } else {
      for matcher in with.values() {
        if !matcher.match_when_empty().await? {
          return Ok(false);
        }
      }
    }
  }

  matched_rules.push(&module_rule.effect);
  if let Some(pattern) = &module_rule.r#as {
    apply_rule_as(pattern, resource_data.to_mut());
  }

  if let Some(rules) = &module_rule.rules {
    module_rules_matcher_async(rules, ctx, resource_data, matched_rules).await?;
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      let original_resource_data = rule.one_of.as_ref().map(|_| resource_data.clone());
      if module_rule_matcher(rule, ctx, resource_data, matched_rules).await? {
        matched_once = true;
        break;
      }
      if let Some(original_resource_data) = original_resource_data {
        *resource_data = original_resource_data;
      }
    }
    if !matched_once {
      return Ok(false);
    }
  }

  Ok(true)
}
