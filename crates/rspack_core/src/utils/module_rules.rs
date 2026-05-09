use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_loader_runner::ResourceData;
use rspack_paths::Utf8Path;

use crate::{
  DataRef, DependencyCategory, ImportAttributes, ModuleRule, ModuleRuleEffect,
  RuleSetConditionWithEmpty,
};

pub async fn module_rules_matcher<'a>(
  rules: &'a [ModuleRule],
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Result<()> {
  let matched_rules_len = matched_rules.len();
  if let Some(result) = module_rules_matcher_sync(
    rules,
    resource_data,
    issuer,
    issuer_layer,
    dependency,
    attributes,
    matched_rules,
  ) {
    return result;
  }
  matched_rules.truncate(matched_rules_len);
  module_rules_matcher_async(
    rules,
    resource_data,
    issuer,
    issuer_layer,
    dependency,
    attributes,
    matched_rules,
  )
  .await
}

fn module_rules_matcher_sync<'a>(
  rules: &'a [ModuleRule],
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Option<Result<()>> {
  for rule in rules {
    match module_rule_matcher_sync(
      rule,
      resource_data,
      issuer,
      issuer_layer,
      dependency,
      attributes,
      matched_rules,
    ) {
      Some(Ok(_)) => {}
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  }
  Some(Ok(()))
}

async fn module_rules_matcher_async<'a>(
  rules: &'a [ModuleRule],
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Result<()> {
  for rule in rules {
    module_rule_matcher_async(
      rule,
      resource_data,
      issuer,
      issuer_layer,
      dependency,
      attributes,
      matched_rules,
    )
    .await?;
  }
  Ok(())
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
pub async fn module_rule_matcher<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Result<bool> {
  let matched_rules_len = matched_rules.len();
  if let Some(result) = module_rule_matcher_sync(
    module_rule,
    resource_data,
    issuer,
    issuer_layer,
    dependency,
    attributes,
    matched_rules,
  ) {
    return result;
  }
  matched_rules.truncate(matched_rules_len);
  module_rule_matcher_async(
    module_rule,
    resource_data,
    issuer,
    issuer_layer,
    dependency,
    attributes,
    matched_rules,
  )
  .await
}

fn module_rule_matcher_sync<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Option<Result<bool>> {
  if let Some(test_rule) = &module_rule.rspack_resource {
    ensure_sync_matched!(test_rule.try_match_sync(resource_data.resource().into()));
  }

  let resource_path = resource_data
    .path()
    .unwrap_or_else(|| Utf8Path::new(""))
    .as_str();

  if let Some(test_rule) = &module_rule.test {
    ensure_sync_matched!(test_rule.try_match_sync(resource_path.into()));
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
    ensure_sync_matched!(check_optional_sync(issuer_rule, issuer.map(Into::into)));
  }

  if let Some(issuer_layer_rule) = &module_rule.issuer_layer {
    ensure_sync_matched!(check_optional_sync(
      issuer_layer_rule,
      issuer_layer.map(Into::into),
    ));
  }

  if let Some(dependency_rule) = &module_rule.dependency {
    ensure_sync_matched!(dependency_rule.try_match_sync(dependency.as_str().into()));
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
    if let Some(attributes) = attributes {
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

  if let Some(rules) = &module_rule.rules {
    match module_rules_matcher_sync(
      rules,
      resource_data,
      issuer,
      issuer_layer,
      dependency,
      attributes,
      matched_rules,
    ) {
      Some(Ok(())) => {}
      Some(Err(err)) => return Some(Err(err)),
      None => return None,
    }
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      match module_rule_matcher_sync(
        rule,
        resource_data,
        issuer,
        issuer_layer,
        dependency,
        attributes,
        matched_rules,
      ) {
        Some(Ok(true)) => {
          matched_once = true;
          break;
        }
        Some(Ok(false)) => {}
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
async fn module_rule_matcher_async<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
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
    && !check_optional_async(issuer_rule, issuer.map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(issuer_layer_rule) = &module_rule.issuer_layer
    && !check_optional_async(issuer_layer_rule, issuer_layer.map(Into::into)).await?
  {
    return Ok(false);
  }

  if let Some(dependency_rule) = &module_rule.dependency
    && !dependency_rule
      .try_match(dependency.as_str().into())
      .await?
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
    if let Some(attributes) = attributes {
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

  if let Some(rules) = &module_rule.rules {
    module_rules_matcher(
      rules,
      resource_data,
      issuer,
      issuer_layer,
      dependency,
      attributes,
      matched_rules,
    )
    .await?;
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      if module_rule_matcher(
        rule,
        resource_data,
        issuer,
        issuer_layer,
        dependency,
        attributes,
        matched_rules,
      )
      .await?
      {
        matched_once = true;
        break;
      }
    }
    if !matched_once {
      return Ok(false);
    }
  }

  Ok(true)
}
