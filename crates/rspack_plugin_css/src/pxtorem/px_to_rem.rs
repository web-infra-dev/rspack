use rustc_hash::FxHashMap as HashMap;
use swc_core::css::{
  ast::{ComponentValue, Declaration, DimensionToken, Token},
  codegen::{
    writer::basic::{BasicCssWriter, BasicCssWriterConfig},
    CodeGenerator, CodegenConfig, Emit,
  },
  visit::{VisitMut, VisitMutWith},
};
use swc_core::{common::DUMMY_SP, ecma::atoms::Atom};

use super::{
  filter_prop_list::{
    contain, ends_with, exact, not_contain, not_ends_with, not_exact, not_starts_with, starts_with,
  },
  options::PxToRemOptions,
};

impl From<PxToRemOptions> for PxToRem {
  fn from(option: PxToRemOptions) -> Self {
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
      skip_mutate_length: false,
      mutated: false,
    };

    // https://github.com/cuth/postcss-pxtorem/blob/master/index.js#L25-L44
    if ret.prop_list.is_empty() {
      ret.prop_list = vec!["*".to_string()];
    }

    ret.normalize_options();
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
  has_wild: bool,
  pub match_list: MatchList,
  all_match: bool,
  map_stack: Vec<HashMap<Atom, u32>>,
  skip_mutate_length: bool,
  /// Flag to mark if declaration has been mutated
  mutated: bool,
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

  /// Checking if the prop match against any pattern of `prop_list`
  /// Related logic you could reference https://github.com/cuth/postcss-pxtorem/blob/master/index.js#L89-L116
  fn is_match(&self, prop: &str) -> bool {
    if self.all_match {
      return true;
    };
    (self.has_wild
      || self.match_list.exact_list.iter().any(|p| p == prop)
      || self
        .match_list
        .contain_list
        .iter()
        .any(|p| prop.contains(p))
      || self
        .match_list
        .starts_with_list
        .iter()
        .any(|p| prop.starts_with(p))
      || self
        .match_list
        .ends_with_list
        .iter()
        .any(|p| prop.ends_with(p)))
      && !(self.match_list.not_exact_list.iter().any(|p| p == prop)
        || self
          .match_list
          .not_contain_list
          .iter()
          .any(|p| prop.contains(p))
        || self
          .match_list
          .not_starts_list
          .iter()
          .any(|p| prop.starts_with(p))
        || self
          .match_list
          .not_ends_list
          .iter()
          .any(|p| prop.ends_with(p)))
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

impl VisitMut for PxToRem {
  fn visit_mut_at_rule(&mut self, n: &mut swc_core::css::ast::AtRule) {
    if self.media_query {
      if let Some(ref mut prelude) = n.prelude {
        self.visit_mut_at_rule_prelude(prelude);
      }
    }
    if let Some(ref mut block) = n.block {
      self.visit_mut_simple_block(block);
    }
  }

  fn visit_mut_rule(&mut self, n: &mut swc_core::css::ast::Rule) {
    match n {
      swc_core::css::ast::Rule::QualifiedRule(rule) => {
        // Reducing codegen overhead if there are no selector_black_list
        if !self.selector_black_list.is_empty() {
          let mut selector_string = String::new();
          let wr = BasicCssWriter::new(
            &mut selector_string,
            None, // Some(&mut src_map_buf),
            BasicCssWriterConfig::default(),
          );
          let mut gen = CodeGenerator::new(wr, CodegenConfig { minify: false });
          gen.emit(&rule.prelude).expect("TODO:");
          if !self.black_listed_selector(&selector_string) {
            rule.visit_mut_with(self);
          }
        } else {
          rule.visit_mut_with(self);
        }
      }
      swc_core::css::ast::Rule::AtRule(at) => at.visit_mut_with(self),
      swc_core::css::ast::Rule::ListOfComponentValues(comp) => comp.visit_mut_with(self),
    }
  }

  fn visit_mut_simple_block(&mut self, n: &mut swc_core::css::ast::SimpleBlock) {
    // We push a declaration map into map_stack before enter the block,
    // pop it before leave the block. The hashmap record the relation how many times
    // each declaration name occurs.

    // The original implementation you could reference here, https://github.com/cuth/postcss-pxtorem/blob/122649015322214f8e9d1ac852eb11c0791b634b/index.js#L164
    // There is no easy way we could do the same thing in `swc_core::css`, except we made the trade off to perf which is we could codegen each prop and value of declaration
    // That means we almost codegen twice for each css file when postcss plugin is enable.
    let mut map: HashMap<Atom, u32> = HashMap::default();
    // prescan
    for ele in n.value.iter_mut() {
      if let swc_core::css::ast::ComponentValue::Declaration(decl) = ele {
        let name = get_decl_name(decl);
        *(map.entry(name).or_insert(0)) += 1;
      }
    }
    self.map_stack.push(map);

    // only used for `replace = false`
    let mut snapshot_and_index_list: Vec<(usize, ComponentValue)> = vec![];

    for (index, ele) in n.value.iter_mut().enumerate() {
      if let ComponentValue::Declaration(decl) = ele {
        let snapshot = if self.replace {
          None
        } else {
          Some(decl.clone())
        };
        self.mutated = false;
        self.visit_mut_declaration(decl);

        if !self.replace && self.mutated {
          // SAFETY: if `self.replace = false` we must save the snapshot of the declaration
          let mut snapshot = snapshot.expect("TODO:");
          std::mem::swap(decl, &mut snapshot);
          // Now, snapshot save the mutated version of declaration
          snapshot_and_index_list.push((index, ComponentValue::Declaration(snapshot)));
          // Next, we will insert the mutated version of declaration after the original version of declaration
        }
        self.mutated = false;
      } else {
        ele.visit_mut_with(self);
      }
    }

    if !self.replace {
      // Why reverse order? Insertion will mutate the vector and the order of original element would change
      // e.g.
      // We have array [1, 2, 3];
      // We want insert mutated version of each element after the original element, expect [1, 1', 2, 2', 3, 3']
      // the `snapshot_and_index_list` would be `[(0, 1'), (1, '2'), (2, 3')]
      // If insert in order
      // the result would be
      // [1, 1',2',3', 2, 3]
      // Insert in reverse order
      // [1, 1', 2,2', 3, 3']
      for (index, component) in snapshot_and_index_list.into_iter().rev() {
        n.value.insert(index + 1, component);
      }
    }
    self.map_stack.pop();
  }

  fn visit_mut_declaration(&mut self, n: &mut swc_core::css::ast::Declaration) {
    let map = self.map_stack.last().expect("TODO:");
    let name = get_decl_name(n);
    let frequency = *map.get(&name).expect("TODO:");

    if !self.is_match(&name) {
      return;
    }

    for ele in n.value.iter_mut() {
      match ele {
        ComponentValue::Dimension(box d) => {
          if let swc_core::css::ast::Dimension::Length(len) = d {
            self.skip_mutate_length = frequency != 1;
            self.visit_mut_length(len);
            self.skip_mutate_length = false;
          }
        }
        ComponentValue::PreservedToken(box tok) => {
          if let Token::Dimension(box DimensionToken {
            unit,
            value,
            raw_unit,
            raw_value,
            ..
          }) = &mut tok.token
          {
            if unit == "px" && frequency == 1 {
              if *value != 0f64 && (*value).abs() >= self.min_pixel_value {
                self.mutated = true;
                *unit = "rem".into();
                *value = self.normalized_num(*value);
                *raw_unit = Atom::from(unit.to_string());
                *raw_value = value.to_string().into();
              } else if *value == 0f64 {
                self.mutated = true;
                *unit = "".into();
                *raw_unit = Atom::from(unit.to_string());
              }
            }
          }
        }
        _ => self.visit_mut_component_value(ele),
      }
    }
  }

  fn visit_mut_length(&mut self, len: &mut swc_core::css::ast::Length) {
    if let Some(ref raw) = len.unit.raw && raw == "px" && !self.skip_mutate_length {
      if len.value.value != 0f64 && len.value.value.abs() >= self.min_pixel_value {
        self.mutated = true;
        let normalized_value = self.normalized_num(len.value.value);
        len.unit.span = DUMMY_SP;
        len.unit.raw = Some("rem".into());
        len.unit.value = "rem".into();
        len.value.span = DUMMY_SP;
        len.value.raw = Some(normalized_value.to_string().into());
        len.value.value = normalized_value;
        // xxx: 0px;
      } else if len.value.value == 0f64 {
        self.mutated = true;
        len.unit.span = DUMMY_SP;
        len.unit.raw = None;
        len.unit.value = "".into();
      }
    }
  }
}

pub fn px_to_rem(option: PxToRemOptions) -> impl VisitMut {
  PxToRem::from(option)
}

fn get_decl_name(n: &Declaration) -> Atom {
  match &n.name {
    swc_core::css::ast::DeclarationName::Ident(indent) => indent
      .raw
      .clone()
      .unwrap_or_else(|| Atom::from(indent.value.to_string())),
    swc_core::css::ast::DeclarationName::DashedIdent(indent) => indent
      .raw
      .clone()
      .unwrap_or_else(|| Atom::from(indent.value.to_string())),
  }
}
