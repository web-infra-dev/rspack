use rspack_error::{internal_error, Result};
use rspack_loader_runner::ResourceData;

use crate::{ModuleRule, ModuleRuleCondition};

/// Match the condition against the given `data`. Returns `true` if the condition matches.
pub fn module_rule_matcher_condition(condition: &ModuleRuleCondition, data: &str) -> bool {
  match condition {
    ModuleRuleCondition::String(s) => data.starts_with(s),
    ModuleRuleCondition::Regexp(r) => r.test(data),
  }
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
pub fn module_rule_matcher<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&str>,
) -> Result<Option<&'a ModuleRule>> {
  if module_rule.test.is_none()
    && module_rule.resource.is_none()
    && module_rule.resource_query.is_none()
    && module_rule.include.is_none()
    && module_rule.exclude.is_none()
    && module_rule.issuer.is_none()
    && module_rule.one_of.is_none()
  {
    return Err(internal_error!(
      "ModuleRule must have at least one condition"
    ));
  }

  return module_rule_matcher_inner(module_rule, resource_data, issuer);
}

pub fn module_rule_matcher_inner<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&str>,
) -> Result<Option<&'a ModuleRule>> {
  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test {
    if !module_rule_matcher_condition(test_rule, &resource_data.resource_path.to_string_lossy()) {
      return Ok(None);
    }
  } else if let Some(resource_rule) = &module_rule.resource {
    if !module_rule_matcher_condition(
      resource_rule,
      &resource_data.resource_path.to_string_lossy(),
    ) {
      return Ok(None);
    }
  }

  if let Some(include_rule) = &module_rule.include {
    if include_rule.iter().all(|rule| {
      !module_rule_matcher_condition(rule, &resource_data.resource_path.to_string_lossy())
    }) {
      return Ok(None);
    }
  }

  if let Some(exclude_rule) = &module_rule.exclude {
    if exclude_rule.iter().any(|rule| {
      module_rule_matcher_condition(rule, &resource_data.resource_path.to_string_lossy())
    }) {
      return Ok(None);
    }
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = &resource_data.resource_query {
      if !module_rule_matcher_condition(resource_query_rule, resource_query) {
        return Ok(None);
      }
    } else {
      return Ok(None);
    }
  }

  if let Some(issuer_options) = &module_rule.issuer
    && let Some(not_issuer) = &issuer_options.not
    && let Some(issuer) = issuer
    && not_issuer
      .iter()
      .any(|i| module_rule_matcher_condition(i, issuer))
  {
    return Ok(None);
  }

  if let Some(one_of) = &module_rule.one_of {
    let matched_rule = one_of.iter().find_map(|module_rule| {
      match module_rule_matcher_inner(module_rule, resource_data, issuer) {
        Ok(val) => val,
        Err(_err) => None,
      }
    });

    return Ok(matched_rule);
  }

  Ok(Some(module_rule))
}
