use swc_common::DUMMY_SP;
use swc_css::{
  ast::{ComponentValue, Token},
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  visit::{VisitMut, VisitMutWith, VisitWith},
};

use super::{
  filter_prop_list::{
    contain, ends_with, exact, not_contain, not_ends_with, not_exact, not_starts_with, starts_with,
  },
  option::PxToRemOption,
};

impl From<PxToRemOption> for PxToRem {
  fn from(option: PxToRemOption) -> Self {
    let mut ret = PxToRem {
      root_value: option.root_value.unwrap_or(16u32),
      unit_precision: option.unit_precision.unwrap_or(5u32),
      selector_black_list: option.selector_black_list.unwrap_or_default(),
      prop_list: option.prop_list.unwrap_or_else(|| {
        vec![
          "font".to_string(),
          "font-size".to_string(),
          "line-height".to_string(),
          "letter-spacing".to_string(),
        ]
      }),
      replace: option.replace.unwrap_or(true),
      media_query: option.media_query.unwrap_or(false),
      min_pixel_value: option.min_pixel_value.unwrap_or(0f64),
      has_wild: false,
      match_list: MatchList::default(),
      all_match: false,
      map_stack: vec![],
    };

    // https://github.com/cuth/postcss-pxtorem/blob/master/index.js#L25-L44
    if ret.prop_list.is_empty() {
      ret.prop_list = vec!["*".to_string()];
    }

    ret.normalize_options();
    dbg!(&ret.match_list);
    ret
  }
}
#[derive(Debug)]
pub struct PxToRem {
  root_value: u32,
  unit_precision: u32,
  selector_black_list: Vec<String>,
  prop_list: Vec<String>,
  replace: bool,
  media_query: bool,
  min_pixel_value: f64,
  has_wild: bool, // exclude: null we don't need the prop, since this is always used for cli
  pub match_list: MatchList,
  // exact_list: Vec<&'a String>,
  all_match: bool,
  map_stack: Vec<Vec<(String, String)>>,
}

impl PxToRem {
  pub fn normalize_options(&mut self) {
    self.match_list = MatchList {
      exact_list: exact(&self.prop_list),
      contain_list: contain(&self.prop_list),
      starts_with_list: starts_with(&self.prop_list),
      ends_with_list: ends_with(&self.prop_list),
      not_exact_list: not_exact(&self.prop_list),
      not_contain_list: not_contain(&self.prop_list),
      not_starts_list: not_starts_with(&self.prop_list),
      not_ends_list: not_ends_with(&self.prop_list),
    };
    let has_wild = self.prop_list.iter().any(|prop| prop == "*");
    let all_match = has_wild && self.prop_list.len() == 1;
    self.has_wild = has_wild;
    self.all_match = all_match;
  }

  fn normalized_num(&self, n: f64) -> f64 {
    let num = n.abs();
    let sign = n.signum();
    let normalized_num = if num < self.min_pixel_value {
      num
    } else {
      let fixed_value = num / self.root_value as f64;
      if fixed_value == 0f64 {
        fixed_value
      } else {
        let format_fixed_float = format!("{:.*}", self.unit_precision as usize, fixed_value);
        let cont = format_fixed_float.ends_with('0');
        if cont {
          let mut temp = format_fixed_float.trim_end_matches('0');
          if temp.ends_with('.') {
            temp = &temp[0..temp.len() - 1];
          }
          temp.parse::<f64>().unwrap_or(num)
        } else {
          format_fixed_float.parse::<f64>().unwrap_or(num)
        }
      }
    };
    normalized_num * sign
  }

  /// TODO: Suppport regex pattern
  /// Regex supporting is none trivial, only support string pattern for now, main reasons are listed below:
  /// 1. Difference between ECMA regex and rust `regex` crate, some features of `ECMA regex` are not supported by `regex`
  /// 2. Hard to test, there is no way to represent a regex in json file
  /// 3. Our node binding testing system are not stable yet.
  fn black_listed_selector(&self, selector: &str) -> bool {
    self
      .selector_black_list
      .iter()
      .any(|pattern| selector.contains(pattern))
  }

  fn is_match(&self, prop: &str) -> bool {
    if self.all_match {
      return true;
    };
    (self.has_wild
      || self
        .match_list
        .exact_list
        .iter()
        .any(|p| p.as_str() == prop)
      || self
        .match_list
        .contain_list
        .iter()
        .any(|p| prop.contains(p.as_str()))
      || self
        .match_list
        .starts_with_list
        .iter()
        .any(|p| prop.starts_with(p.as_str()))
      || self
        .match_list
        .ends_with_list
        .iter()
        .any(|p| prop.ends_with(p.as_str())))
      && !(self.match_list.not_exact_list.iter().any(|p| p == prop)
        || self
          .match_list
          .not_contain_list
          .iter()
          .any(|p| prop.contains(p.as_str()))
        || self
          .match_list
          .not_starts_list
          .iter()
          .any(|p| prop.starts_with(p.as_str()))
        || self
          .match_list
          .not_ends_list
          .iter()
          .any(|p| prop.ends_with(p.as_str())))
  }
}

#[derive(Default, Debug)]
pub struct MatchList {
  pub exact_list: Vec<String>,
  pub contain_list: Vec<String>,
  pub starts_with_list: Vec<String>,
  pub ends_with_list: Vec<String>,
  pub not_exact_list: Vec<String>,
  pub not_contain_list: Vec<String>,
  pub not_starts_list: Vec<String>,
  pub not_ends_list: Vec<String>,
}
// use swc_css::{ast::ComponentValue, visit::VisitMut};

struct HasMatchedRuleName {
  matched: bool,
}

impl VisitMut for PxToRem {
  fn visit_mut_at_rule(&mut self, n: &mut swc_css::ast::AtRule) {
    if self.media_query {
      if let Some(ref mut prelude) = n.prelude {
        self.visit_mut_at_rule_prelude(prelude);
      }
    }
    if let Some(ref mut block) = n.block {
      self.visit_mut_simple_block(block);
    }
  }

  fn visit_mut_rule(&mut self, n: &mut swc_css::ast::Rule) {
    match n {
      swc_css::ast::Rule::QualifiedRule(rule) => {
        let mut selector = String::new();
        let wr = BasicCssWriter::new(
          &mut selector,
          None, // Some(&mut src_map_buf),
          BasicCssWriterConfig::default(),
        );
        let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });
        gen.emit(&rule.prelude).unwrap();
        if !self.black_listed_selector(&selector) {
          self.visit_mut_simple_block(&mut rule.block);
        }
      }
      swc_css::ast::Rule::Invalid(_) | swc_css::ast::Rule::AtRule(_) => n.visit_mut_with(self),
    }
  }
  fn visit_mut_declaration(&mut self, n: &mut swc_css::ast::Declaration) {
    let name = match &n.name {
      swc_css::ast::DeclarationName::Ident(indent) => indent.value.clone(),
      swc_css::ast::DeclarationName::DashedIdent(indent) => indent.value.clone(),
    };

    if !self.is_match(&name) {
      return;
    }

    for ele in n.value.iter_mut() {
      match ele {
        ComponentValue::Dimension(d) => match d {
          swc_css::ast::Dimension::Length(len) => {
            self.visit_mut_length(len);
            // let num = l.value.clone();
          }
          swc_css::ast::Dimension::Angle(_)
          | swc_css::ast::Dimension::Time(_)
          | swc_css::ast::Dimension::Frequency(_)
          | swc_css::ast::Dimension::Resolution(_)
          | swc_css::ast::Dimension::Flex(_)
          | swc_css::ast::Dimension::UnknownDimension(_) => {}
        },
        ComponentValue::PreservedToken(tok) => match &mut tok.token {
          Token::Dimension {
            unit,
            value,
            raw_unit,
            raw_value,
            ..
          } => {
            if unit == "px" && *value != 0f64 {
              *unit = "rem".into();
              *value = self.normalized_num(*value);
              *raw_unit = unit.clone();
              *raw_value = value.to_string().into();
            }
          }
          _ => {}
        },
        _ => self.visit_mut_component_value(ele),
      }
    }
  }

  fn visit_mut_length(&mut self, len: &mut swc_css::ast::Length) {
    if &len.unit.value == "px" && len.value.value != 0f64 {
      len.unit.span = DUMMY_SP;
      // TODO: figure it out
      len.unit.raw = None;
      len.unit.value = "rem".into();
      len.value.span = DUMMY_SP;
      // TODO: figure out what the raw is;
      len.value.raw = None;
      len.value.value = self.normalized_num(len.value.value);
      // len.value.raw =
    }
  }
}

pub fn px_to_rem(option: PxToRemOption) -> impl VisitMut {
  PxToRem::from(option)
}
