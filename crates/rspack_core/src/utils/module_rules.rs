use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_loader_runner::ResourceData;
use rspack_paths::Utf8Path;

use crate::{DependencyCategory, ImportAttributes, ImportPhase, ModuleRule, ModuleRuleEffect};

pub struct MatchContext<'ctx> {
  pub(crate) resource_data: &'ctx ResourceData,
  pub(crate) issuer: Option<&'ctx str>,
  pub(crate) issuer_layer: Option<&'ctx str>,
  pub(crate) dependency: DependencyCategory,
  pub(crate) phase: ImportPhase,
  pub(crate) attributes: Option<&'ctx ImportAttributes>,
}

pub async fn module_rules_matcher<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  ctx: &MatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<()> {
  for rule in rules {
    module_rule_matcher(rule, ctx, matched_rules).await?;
  }
  Ok(())
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
#[async_recursion]
pub async fn module_rule_matcher<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  ctx: &MatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  let resource_data = ctx.resource_data;

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

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = resource_data.query() {
      if !resource_query_rule.try_match(resource_query.into()).await? {
        return Ok(false);
      }
    } else if !resource_query_rule.match_when_empty().await? {
      return Ok(false);
    }
  }

  if let Some(resource_fragment_condition) = &module_rule.resource_fragment {
    if let Some(resource_fragment) = resource_data.fragment() {
      if !resource_fragment_condition
        .try_match(resource_fragment.into())
        .await?
      {
        return Ok(false);
      }
    } else if !resource_fragment_condition.match_when_empty().await? {
      return Ok(false);
    }
  }

  if let Some(mimetype_condition) = &module_rule.mimetype {
    if let Some(mimetype) = resource_data.mimetype() {
      if !mimetype_condition.try_match(mimetype.into()).await? {
        return Ok(false);
      }
    } else if !mimetype_condition.match_when_empty().await? {
      return Ok(false);
    }
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

  if let Some(issuer_rule) = &module_rule.issuer {
    match ctx.issuer {
      Some(issuer) => {
        if !issuer_rule.try_match(issuer.into()).await? {
          return Ok(false);
        }
      }
      None => {
        if !issuer_rule.match_when_empty().await? {
          return Ok(false);
        }
      }
    }
  }

  if let Some(issuer_layer_rule) = &module_rule.issuer_layer {
    match ctx.issuer_layer {
      Some(issuer_layer) => {
        if !issuer_layer_rule.try_match(issuer_layer.into()).await? {
          return Ok(false);
        }
      }
      None => {
        if !issuer_layer_rule.match_when_empty().await? {
          return Ok(false);
        }
      }
    };
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
        if let Some(v) = k
          .split('.')
          .try_fold(resource_description.json(), |acc, key| acc.get(key))
        {
          if !matcher.try_match(v.into()).await? {
            return Ok(false);
          }
        } else if !matcher.match_when_empty().await? {
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
        if let Some(v) = attributes.get(k) {
          if !matcher.try_match(v.into()).await? {
            return Ok(false);
          }
        } else if !matcher.match_when_empty().await? {
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
    module_rules_matcher(rules, ctx, matched_rules).await?;
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      if module_rule_matcher(rule, ctx, matched_rules).await? {
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
