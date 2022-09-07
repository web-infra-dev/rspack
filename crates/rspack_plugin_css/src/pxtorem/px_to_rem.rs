#[derive(Default)]
pub struct PxToRemOption {
  pub root_value: Option<i32>,
  pub unit_precision: Option<i32>,
  pub selector_black_list: Option<Vec<String>>,
  pub prop_list: Option<Vec<String>>,
  pub replace: Option<bool>,
  pub media_query: Option<bool>,
  pub min_pixel_value: Option<f64>,
}

impl From<PxToRemOption> for PxtToRem {
  fn from(option: PxToRemOption) -> Self {}
}
#[derive(Debug)]
pub struct PxtToRem {
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
  map_stack: Vec<Vec<(SmolStr, SmolStr)>>,
}

// use swc_css::{ast::ComponentValue, visit::VisitMut};
// struct PxToRem {}

// impl VisitMut for PxToRem {
//   fn visit_mut_declaration(&mut self, n: &mut swc_css::ast::Declaration) {
//     let name = match &n.name {
//       swc_css::ast::DeclarationName::Ident(indent) => indent.value.clone(),
//       swc_css::ast::DeclarationName::DashedIdent(indent) => indent.value.clone(),
//     };
//     for ele in n.value.iter_mut() {
//       match ele {
//         ComponentValue::Dimension(d) => match d {
//           swc_css::ast::Dimension::Length(len) => {
//             // let num = l.value.clone();
//             // if &len.unit.value.clone() == "px" {
//             //   len.unit.value = "rem".into();
//             // }
//           }
//           swc_css::ast::Dimension::Angle(_) => todo!(),
//           swc_css::ast::Dimension::Time(_) => todo!(),
//           swc_css::ast::Dimension::Frequency(_) => todo!(),
//           swc_css::ast::Dimension::Resolution(_) => todo!(),
//           swc_css::ast::Dimension::Flex(_) => todo!(),
//           swc_css::ast::Dimension::UnknownDimension(_) => todo!(),
//         },
//         _ => {}
//       }
//     }
//   }

//   fn visit_mut_length(&mut self, n: &mut swc_css::ast::Length) {}
// }
