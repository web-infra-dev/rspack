use rspack_error::{internal_error, Result};
use rspack_loader_runner::ResourceData;

use crate::ModuleRule;

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

  Ok(module_rule_matcher_inner(
    module_rule,
    resource_data,
    issuer,
  ))
}

pub fn module_rule_matcher_inner<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&str>,
) -> Option<&'a ModuleRule> {
  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test
    && !test_rule.is_match(&resource_data.resource_path.to_string_lossy()) {
    return None;
  } else if let Some(resource_rule) = &module_rule.resource
    && !resource_rule.is_match(&resource_data.resource_path.to_string_lossy()) {
    return None;
  }

  if let Some(include_rule) = &module_rule.include
    && !include_rule.is_match(&resource_data.resource_path.to_string_lossy()) {
    return None;
  }

  if let Some(exclude_rule) = &module_rule.exclude
    && exclude_rule.is_match(&resource_data.resource_path.to_string_lossy()) {
    return None;
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = &resource_data.resource_query {
      if !resource_query_rule.is_match(resource_query) {
        return None;
      }
    } else {
      return None;
    }
  }

  if let Some(issuer_rule) = &module_rule.issuer
    && let Some(issuer) = issuer
    && !issuer_rule.is_match(issuer) {
    return None;
  }

  if let Some(one_of) = &module_rule.one_of {
    return one_of
      .iter()
      .find_map(|module_rule| module_rule_matcher_inner(module_rule, resource_data, issuer));
  }

  Some(module_rule)
}
