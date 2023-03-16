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

  module_rule_matcher_inner(module_rule, resource_data, issuer)
}

pub fn module_rule_matcher_inner<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&str>,
) -> Result<Option<&'a ModuleRule>> {
  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test
    && !test_rule.try_match(&resource_data.resource_path.to_string_lossy())? {
    return Ok(None);
  } else if let Some(resource_rule) = &module_rule.resource
    && !resource_rule.try_match(&resource_data.resource_path.to_string_lossy())? {
    return Ok(None);
  }

  if let Some(include_rule) = &module_rule.include
    && !include_rule.try_match(&resource_data.resource_path.to_string_lossy())? {
    return Ok(None);
  }

  if let Some(exclude_rule) = &module_rule.exclude
    && exclude_rule.try_match(&resource_data.resource_path.to_string_lossy())? {
    return Ok(None);
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = &resource_data.resource_query {
      if !resource_query_rule.try_match(resource_query)? {
        return Ok(None);
      }
    } else {
      return Ok(None);
    }
  }

  if let Some(issuer_rule) = &module_rule.issuer
    && let Some(issuer) = issuer
    && !issuer_rule.try_match(issuer)? {
    return Ok(None);
  }

  if let Some(one_of) = &module_rule.one_of {
    for rule in one_of {
      if let Some(rule) = module_rule_matcher_inner(rule, resource_data, issuer)? {
        return Ok(Some(rule));
      }
    }
  }

  Ok(Some(module_rule))
}
