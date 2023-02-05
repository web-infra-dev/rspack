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
pub fn module_rule_matcher(
  module_rule: &ModuleRule,
  resource_data: &ResourceData,
  issuer: &str,
) -> Result<bool> {
  // Internal function to match the condition against the given `data`.
  if let Some(func) = &module_rule.func__ {
    match func(resource_data) {
      Ok(result) => return Ok(result),
      Err(e) => return Err(e.into()),
    }
  }

  if module_rule.test.is_none()
    && module_rule.resource.is_none()
    && module_rule.resource_query.is_none()
    && module_rule.include.is_none()
    && module_rule.exclude.is_none()
    && module_rule.issuer.is_none()
  {
    return Err(internal_error!(
      "ModuleRule must have at least one condition"
    ));
  }

  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test {
    if !module_rule_matcher_condition(test_rule, &resource_data.resource_path.to_string_lossy()) {
      return Ok(false);
    }
  } else if let Some(resource_rule) = &module_rule.resource {
    if !module_rule_matcher_condition(
      resource_rule,
      &resource_data.resource_path.to_string_lossy(),
    ) {
      return Ok(false);
    }
  }

  if let Some(include_rule) = &module_rule.include {
    if include_rule.iter().all(|rule| {
      !module_rule_matcher_condition(rule, &resource_data.resource_path.to_string_lossy())
    }) {
      return Ok(false);
    }
  }

  if let Some(exclude_rule) = &module_rule.exclude {
    if exclude_rule.iter().any(|rule| {
      module_rule_matcher_condition(rule, &resource_data.resource_path.to_string_lossy())
    }) {
      return Ok(false);
    }
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = &resource_data.resource_query {
      if !module_rule_matcher_condition(resource_query_rule, resource_query) {
        return Ok(false);
      }
    } else {
      return Ok(false);
    }
  }

  if let Some(issuer_options) = &module_rule.issuer {
    if let Some(not_issuer) = &issuer_options.not {
      if not_issuer
        .iter()
        .any(|i| module_rule_matcher_condition(i, issuer))
      {
        return Ok(false);
      }
    }
  }

  Ok(true)
}
