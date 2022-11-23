use rspack_error::Result;
use rspack_loader_runner::ResourceData;

use crate::{ModuleRule, ModuleRuleCondition};

/// Match the condition against the given `data`. Returns `true` if the condition matches.
fn module_rule_matcher_condition(condition: &ModuleRuleCondition, data: &str) -> bool {
  match condition {
    ModuleRuleCondition::String(s) => data.starts_with(s),
    ModuleRuleCondition::Regexp(r) => r.is_match(data),
  }
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
pub fn module_rule_matcher<'r>(
  module_rule: &'r ModuleRule,
  resource_data: &ResourceData,
) -> Option<Result<&'r ModuleRule>> {
  if let Some(func) = &module_rule.func__ {
    match func(resource_data) {
      Ok(result) => {
        if result {
          return Some(Ok(module_rule));
        }

        return None;
      }
      Err(e) => return Some(Err(e.into())),
    }
  }

  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test && module_rule_matcher_condition(test_rule,&resource_data.resource) {
    return Some(Ok(module_rule));
  } else if let Some(resource_rule) = &module_rule.resource && module_rule_matcher_condition(resource_rule,&resource_data.resource) {
    return Some(Ok(module_rule));
  }

  if let Some(resource_query_rule) = &module_rule.resource_query && let Some(resource_query) = &resource_data.resource_query && module_rule_matcher_condition(resource_query_rule,resource_query) {
    return Some(Ok(module_rule));
  }

  None
}
