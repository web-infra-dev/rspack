use rspack_error::Result;
use rspack_loader_runner::ResourceData;
use rspack_paths::Utf8Path;
use smallvec::SmallVec;

use crate::{DependencyCategory, ImportAttributes, ModuleRule, ModuleRuleEffect};

pub async fn module_rules_matcher<'a>(
  rules: &'a [ModuleRule],
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Result<()> {
  let mut stack = SmallVec::<[ModuleRuleMatcherState<'a>; 8]>::with_capacity(8);
  for rule in rules {
    module_rule_matcher_with_stack(
      rule,
      resource_data,
      issuer,
      issuer_layer,
      dependency,
      attributes,
      matched_rules,
      &mut stack,
    )
    .await?;
  }
  Ok(())
}

async fn module_rule_matcher_inner<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
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
    match issuer {
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
    match issuer_layer {
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
      .try_match(dependency.as_str().into())
      .await?
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
    if let Some(attributes) = attributes {
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

  Ok(true)
}

enum ModuleRuleMatcherState<'a> {
  Enter(&'a ModuleRule),
  Rules {
    rules: &'a [ModuleRule],
    index: usize,
  },
  AfterRules {
    one_of: Option<&'a [ModuleRule]>,
  },
  OneOf {
    rules: &'a [ModuleRule],
    index: usize,
    matched_once: bool,
  },
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
async fn module_rule_matcher_with_stack<'a>(
  module_rule: &'a ModuleRule,
  resource_data: &ResourceData,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &DependencyCategory,
  attributes: Option<&ImportAttributes>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
  stack: &mut SmallVec<[ModuleRuleMatcherState<'a>; 8]>,
) -> Result<bool> {
  stack.clear();
  let mut last_rule_result = None;

  stack.push(ModuleRuleMatcherState::Enter(module_rule));

  while let Some(state) = stack.pop() {
    match state {
      ModuleRuleMatcherState::Enter(module_rule) => {
        if !module_rule_matcher_inner(
          module_rule,
          resource_data,
          issuer,
          issuer_layer,
          dependency,
          attributes,
        )
        .await?
        {
          last_rule_result = Some(false);
          continue;
        }

        matched_rules.push(&module_rule.effect);
        stack.push(ModuleRuleMatcherState::AfterRules {
          one_of: module_rule.one_of.as_deref(),
        });

        if let Some(rules) = module_rule.rules.as_deref() {
          stack.push(ModuleRuleMatcherState::Rules { rules, index: 0 });
        }
      }
      ModuleRuleMatcherState::Rules { rules, index } => {
        if index > 0 {
          last_rule_result
            .take()
            .expect("nested rule evaluation should produce a result");
        }

        if let Some(rule) = rules.get(index) {
          stack.push(ModuleRuleMatcherState::Rules {
            rules,
            index: index + 1,
          });
          stack.push(ModuleRuleMatcherState::Enter(rule));
        }
      }
      ModuleRuleMatcherState::AfterRules { one_of } => {
        if let Some(rules) = one_of {
          stack.push(ModuleRuleMatcherState::OneOf {
            rules,
            index: 0,
            matched_once: false,
          });
        } else {
          last_rule_result = Some(true);
        }
      }
      ModuleRuleMatcherState::OneOf {
        rules,
        index,
        matched_once,
      } => {
        let matched_once = if index == 0 {
          matched_once
        } else {
          matched_once
            || last_rule_result
              .take()
              .expect("oneOf branch evaluation should produce a result")
        };

        if matched_once {
          last_rule_result = Some(true);
          continue;
        }

        if let Some(rule) = rules.get(index) {
          stack.push(ModuleRuleMatcherState::OneOf {
            rules,
            index: index + 1,
            matched_once,
          });
          stack.push(ModuleRuleMatcherState::Enter(rule));
        } else {
          last_rule_result = Some(false);
        }
      }
    }
  }

  Ok(last_rule_result.expect("module rule evaluation should always finish with a result"))
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
  let mut stack = SmallVec::<[ModuleRuleMatcherState<'a>; 8]>::with_capacity(8);
  module_rule_matcher_with_stack(
    module_rule,
    resource_data,
    issuer,
    issuer_layer,
    dependency,
    attributes,
    matched_rules,
    &mut stack,
  )
  .await
}

#[cfg(test)]
mod tests {
  use rspack_loader_runner::ResourceData;

  use super::module_rule_matcher;
  use crate::{DependencyCategory, ModuleRule, RuleSetCondition};

  fn rule(layer: &str) -> ModuleRule {
    ModuleRule {
      effect: crate::ModuleRuleEffect {
        layer: Some(layer.to_string()),
        ..Default::default()
      },
      ..Default::default()
    }
  }

  #[tokio::test]
  async fn should_match_rules_before_one_of() {
    let mut root = rule("root");
    root.rules = Some(vec![rule("rules-a"), rule("rules-b")]);

    let mut missed = rule("one-of-missed");
    missed.test = Some(RuleSetCondition::String("/other".to_string()));

    let mut matched = rule("one-of-hit");
    matched.test = Some(RuleSetCondition::String("/root/src".to_string()));
    root.one_of = Some(vec![missed, matched]);

    let resource_data = ResourceData::new_with_resource("/root/src/index.js".to_string());
    let mut matched_rules = Vec::new();

    let result = module_rule_matcher(
      &root,
      &resource_data,
      None,
      None,
      &DependencyCategory::Unknown,
      None,
      &mut matched_rules,
    )
    .await
    .expect("module rule matching should succeed");

    assert!(result);
    assert_eq!(
      matched_rules
        .iter()
        .map(|effect| effect.layer.as_deref())
        .collect::<Vec<_>>(),
      vec![
        Some("root"),
        Some("rules-a"),
        Some("rules-b"),
        Some("one-of-hit")
      ]
    );
  }

  #[tokio::test]
  async fn should_preserve_parent_effect_when_one_of_misses() {
    let mut root = rule("root");
    root.rules = Some(vec![rule("rules-a")]);
    root.one_of = Some(vec![ModuleRule {
      test: Some(RuleSetCondition::String("/other".to_string())),
      ..rule("one-of-missed")
    }]);

    let resource_data = ResourceData::new_with_resource("/root/src/index.js".to_string());
    let mut matched_rules = Vec::new();

    let result = module_rule_matcher(
      &root,
      &resource_data,
      None,
      None,
      &DependencyCategory::Unknown,
      None,
      &mut matched_rules,
    )
    .await
    .expect("module rule matching should succeed");

    assert!(!result);
    assert_eq!(
      matched_rules
        .iter()
        .map(|effect| effect.layer.as_deref())
        .collect::<Vec<_>>(),
      vec![Some("root"), Some("rules-a")]
    );
  }
}
