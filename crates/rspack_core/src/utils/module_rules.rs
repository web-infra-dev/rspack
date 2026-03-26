use async_recursion::async_recursion;
use rspack_error::Result;
use rspack_loader_runner::{DescriptionData as ResourceDescriptionData, ResourceData};
use rspack_paths::Utf8Path;

use crate::{DependencyCategory, ImportAttributes, ModuleRule, ModuleRuleEffect};

pub async fn module_rules_matcher<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  resource_data: &'ctx ResourceData,
  issuer: Option<&'ctx str>,
  issuer_layer: Option<&'ctx str>,
  dependency: &'ctx DependencyCategory,
  attributes: Option<&'ctx ImportAttributes>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<()> {
  let context =
    ModuleRuleMatchContext::new(resource_data, issuer, issuer_layer, dependency, attributes);
  module_rules_matcher_with_context(rules, &context, matched_rules).await
}

/// Match the `ModuleRule` against the given `ResourceData`, and return the matching `ModuleRule` if matched.
pub async fn module_rule_matcher<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  resource_data: &'ctx ResourceData,
  issuer: Option<&'ctx str>,
  issuer_layer: Option<&'ctx str>,
  dependency: &'ctx DependencyCategory,
  attributes: Option<&'ctx ImportAttributes>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  let context =
    ModuleRuleMatchContext::new(resource_data, issuer, issuer_layer, dependency, attributes);
  module_rule_matcher_with_context(module_rule, &context, matched_rules).await
}

struct ModuleRuleMatchContext<'a> {
  resource: &'a str,
  resource_path: &'a str,
  resource_query: Option<&'a str>,
  resource_fragment: Option<&'a str>,
  mimetype: Option<&'a str>,
  scheme: &'a str,
  scheme_is_none: bool,
  issuer: Option<&'a str>,
  issuer_layer: Option<&'a str>,
  dependency: &'a str,
  resource_description: Option<&'a ResourceDescriptionData>,
  attributes: Option<&'a ImportAttributes>,
}

impl<'a> ModuleRuleMatchContext<'a> {
  fn new(
    resource_data: &'a ResourceData,
    issuer: Option<&'a str>,
    issuer_layer: Option<&'a str>,
    dependency: &'a DependencyCategory,
    attributes: Option<&'a ImportAttributes>,
  ) -> Self {
    let scheme = resource_data.get_scheme();
    Self {
      resource: resource_data.resource(),
      resource_path: resource_data
        .path()
        .unwrap_or_else(|| Utf8Path::new(""))
        .as_str(),
      resource_query: resource_data.query(),
      resource_fragment: resource_data.fragment(),
      mimetype: resource_data.mimetype(),
      scheme: scheme.as_str(),
      scheme_is_none: scheme.is_none(),
      issuer,
      issuer_layer,
      dependency: dependency.as_str(),
      resource_description: resource_data.description(),
      attributes,
    }
  }
}

async fn module_rules_matcher_with_context<'rule, 'ctx>(
  rules: &'rule [ModuleRule],
  context: &ModuleRuleMatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<()> {
  for rule in rules {
    module_rule_matcher_with_context(rule, context, matched_rules).await?;
  }
  Ok(())
}

async fn module_rule_matcher_with_context<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  context: &ModuleRuleMatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  if module_rule.rules.is_none() && module_rule.one_of.is_none() {
    return module_rule_matcher_leaf(module_rule, context, matched_rules).await;
  }
  module_rule_matcher_recursive(module_rule, context, matched_rules).await
}

async fn module_rule_matcher_leaf<'rule, 'ctx>(
  module_rule: &'rule ModuleRule,
  context: &ModuleRuleMatchContext<'ctx>,
  matched_rules: &mut Vec<&'rule ModuleRuleEffect>,
) -> Result<bool> {
  if !match_module_rule_conditions(module_rule, context).await? {
    return Ok(false);
  }

  matched_rules.push(&module_rule.effect);
  Ok(true)
}

#[async_recursion]
async fn module_rule_matcher_recursive<'a>(
  module_rule: &'a ModuleRule,
  context: &ModuleRuleMatchContext<'_>,
  matched_rules: &mut Vec<&'a ModuleRuleEffect>,
) -> Result<bool> {
  if !match_module_rule_conditions(module_rule, context).await? {
    return Ok(false);
  }

  matched_rules.push(&module_rule.effect);

  if let Some(rules) = &module_rule.rules {
    module_rules_matcher_with_context(rules, context, matched_rules).await?;
  }

  if let Some(one_of) = &module_rule.one_of {
    let mut matched_once = false;
    for rule in one_of {
      if module_rule_matcher_with_context(rule, context, matched_rules).await? {
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

async fn match_module_rule_conditions(
  module_rule: &ModuleRule,
  context: &ModuleRuleMatchContext<'_>,
) -> Result<bool> {
  if let Some(test_rule) = &module_rule.rspack_resource
    && !test_rule.try_match(context.resource.into()).await?
  {
    return Ok(false);
  }

  // Include all modules that pass test assertion. If you supply a Rule.test option, you cannot also supply a `Rule.resource`.
  // See: https://webpack.js.org/configuration/module/#ruletest
  if let Some(test_rule) = &module_rule.test
    && !test_rule.try_match(context.resource_path.into()).await?
  {
    return Ok(false);
  } else if let Some(resource_rule) = &module_rule.resource
    && !resource_rule
      .try_match(context.resource_path.into())
      .await?
  {
    return Ok(false);
  }

  if let Some(include_rule) = &module_rule.include
    && !include_rule.try_match(context.resource_path.into()).await?
  {
    return Ok(false);
  }

  if let Some(exclude_rule) = &module_rule.exclude
    && exclude_rule.try_match(context.resource_path.into()).await?
  {
    return Ok(false);
  }

  if let Some(resource_query_rule) = &module_rule.resource_query {
    if let Some(resource_query) = context.resource_query {
      if !resource_query_rule.try_match(resource_query.into()).await? {
        return Ok(false);
      }
    } else if !resource_query_rule.match_when_empty().await? {
      return Ok(false);
    }
  }

  if let Some(resource_fragment_condition) = &module_rule.resource_fragment {
    if let Some(resource_fragment) = context.resource_fragment {
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
    if let Some(mimetype) = context.mimetype {
      if !mimetype_condition.try_match(mimetype.into()).await? {
        return Ok(false);
      }
    } else if !mimetype_condition.match_when_empty().await? {
      return Ok(false);
    }
  }

  if let Some(scheme_condition) = &module_rule.scheme {
    if context.scheme_is_none && !scheme_condition.match_when_empty().await? {
      return Ok(false);
    }
    if !scheme_condition.try_match(context.scheme.into()).await? {
      return Ok(false);
    }
  }

  if let Some(issuer_rule) = &module_rule.issuer {
    match context.issuer {
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
    match context.issuer_layer {
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
    && !dependency_rule.try_match(context.dependency.into()).await?
  {
    return Ok(false);
  }

  if let Some(description_data) = &module_rule.description_data {
    if let Some(resource_description) = context.resource_description {
      if !match_description_data(description_data, resource_description).await? {
        return Ok(false);
      }
    } else if !match_when_empty_for_all(description_data.values()).await? {
      return Ok(false);
    }
  }

  if let Some(with) = &module_rule.with {
    if let Some(attributes) = context.attributes {
      for (key, matcher) in with {
        if let Some(value) = attributes.get(key) {
          if !matcher.try_match(value.into()).await? {
            return Ok(false);
          }
        } else if !matcher.match_when_empty().await? {
          return Ok(false);
        }
      }
    } else if !match_when_empty_for_all(with.values()).await? {
      return Ok(false);
    }
  }

  Ok(true)
}

async fn match_description_data<'a>(
  description_data: impl IntoIterator<Item = (&'a String, &'a crate::RuleSetConditionWithEmpty)>,
  resource_description: &ResourceDescriptionData,
) -> Result<bool> {
  let json = resource_description.json();
  for (key, matcher) in description_data {
    if let Some(value) = key
      .split('.')
      .try_fold(json, |acc, segment| acc.get(segment))
    {
      if !matcher.try_match(value.into()).await? {
        return Ok(false);
      }
    } else if !matcher.match_when_empty().await? {
      return Ok(false);
    }
  }
  Ok(true)
}

async fn match_when_empty_for_all<'a>(
  matchers: impl IntoIterator<Item = &'a crate::RuleSetConditionWithEmpty>,
) -> Result<bool> {
  for matcher in matchers {
    if !matcher.match_when_empty().await? {
      return Ok(false);
    }
  }
  Ok(true)
}

#[cfg(test)]
mod tests {
  use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
  };

  use futures::FutureExt;
  use rspack_regex::RspackRegex;

  use super::module_rule_matcher;
  use crate::{DependencyCategory, ModuleRule, ResourceData, RuleSetCondition};

  #[tokio::test]
  async fn supports_one_of_rules_with_async_conditions() {
    let calls = Arc::new(AtomicUsize::new(0));
    let module_rule = ModuleRule {
      one_of: Some(vec![
        ModuleRule {
          test: Some(RuleSetCondition::Func(Box::new({
            let calls = Arc::clone(&calls);
            move |_| {
              let calls = Arc::clone(&calls);
              async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(false)
              }
              .boxed()
            }
          }))),
          ..Default::default()
        },
        ModuleRule {
          test: Some(RuleSetCondition::Regexp(
            RspackRegex::new("\\.js$").expect("regex should compile"),
          )),
          ..Default::default()
        },
      ]),
      ..Default::default()
    };
    let resource_data = ResourceData::new_with_path(
      "/project/src/index.js".into(),
      "/project/src/index.js".into(),
      None,
      None,
    );
    let mut matched_rules = Vec::new();

    let matched = module_rule_matcher(
      &module_rule,
      &resource_data,
      None,
      None,
      &DependencyCategory::Esm,
      None,
      &mut matched_rules,
    )
    .await
    .expect("module rule should match");

    assert!(matched);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(matched_rules.len(), 2);
  }
}
