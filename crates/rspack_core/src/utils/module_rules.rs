use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_loader_runner::ResourceData;

use crate::{DependencyCategory, ModuleRule};

pub async fn module_rules_matcher<'a>(
  rules: &'a [ModuleRule],
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  dependency: &DependencyCategory,
  matched_rules: &mut Vec<&'a ModuleRule>,
) -> Result<()> {
  for rule in rules {
    module_rule_matcher(rule, resource_data, issuer, dependency, matched_rules).await?;
  }
  Ok(())
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
#[async_recursion]
pub async fn module_rule_matcher<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  dependency: &DependencyCategory,
  matched_rules: &mut Vec<&'a ModuleRule>,
) -> Result<bool> {
  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test
    && !test_rule.try_match(&resource_data.resource_path.to_string_lossy()).await? {
    return Ok(false);
  } else if let Some(resource_rule) = &module_rule.resource
    && !resource_rule.try_match(&resource_data.resource_path.to_string_lossy()).await? {
    return Ok(false);
  }

  if let Some(include_rule) = &module_rule.include
    && !include_rule.try_match(&resource_data.resource_path.to_string_lossy()).await? {
    return Ok(false);
  }

  if let Some(exclude_rule) = &module_rule.exclude
    && exclude_rule.try_match(&resource_data.resource_path.to_string_lossy()).await? {
    return Ok(false);
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = &resource_data.resource_query {
      if !resource_query_rule.try_match(resource_query).await? {
        return Ok(false);
      }
    } else {
      return Ok(false);
    }
  }

  if let Some(resource_fragment_condition) = &module_rule.resource_fragment {
    if let Some(resource_fragment) = &resource_data.resource_fragment {
      if !resource_fragment_condition
        .try_match(resource_fragment)
        .await?
      {
        return Ok(false);
      }
    } else {
      return Ok(false);
    }
  }

  if let Some(mimetype_condition) = &module_rule.mimetype {
    if let Some(mimetype) = &resource_data.mimetype {
      if !mimetype_condition.try_match(mimetype).await? {
        return Ok(false);
      }
    } else {
      return Ok(false);
    }
  }

  if let Some(scheme_condition) = &module_rule.scheme {
    let scheme = resource_data.get_scheme();
    if scheme.is_none() {
      return Ok(false);
    }
    if !scheme_condition.try_match(&scheme.to_string()).await? {
      return Ok(false);
    }
  }

  if let Some(issuer_rule) = &module_rule.issuer
    && let Some(issuer) = issuer
    && !issuer_rule.try_match(issuer).await? {
    return Ok(false);
  }

  if let Some(dependency_rule) = &module_rule.dependency
    && !dependency_rule.try_match(&dependency.to_string()).await? {
    return Ok(false);
  }

  if let Some(description_data) = &module_rule.description_data {
    if let Some(resource_description) = &resource_data.resource_description {
      for (k, matcher) in description_data {
        if let Some(v) = resource_description
          .data()
          .raw()
          .get(k)
          .and_then(|v| v.as_str())
        {
          if !matcher.try_match(v).await? {
            return Ok(false);
          }
        } else {
          return Ok(false);
        }
      }
    } else {
      return Ok(false);
    }
  }

  if let Some(one_of) = &module_rule.one_of {
    for rule in one_of {
      if module_rule_matcher(rule, resource_data, issuer, dependency, matched_rules).await? {
        break;
      }
    }
  }

  if let Some(rules) = &module_rule.rules {
    module_rules_matcher(rules, resource_data, issuer, dependency, matched_rules).await?;
  }

  matched_rules.push(module_rule);
  Ok(true)
}
