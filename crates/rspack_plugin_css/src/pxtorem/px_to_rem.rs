use swc_css::{ast::ComponentValue, visit::VisitMut};
struct PxToRem {}

impl VisitMut for PxToRem {
  fn visit_mut_declaration(&mut self, n: &mut swc_css::ast::Declaration) {
    let name = match &n.name {
      swc_css::ast::DeclarationName::Ident(indent) => indent.value.clone(),
      swc_css::ast::DeclarationName::DashedIdent(indent) => indent.value.clone(),
    };
    for ele in n.value.iter_mut() {
      match ele {
        ComponentValue::Dimension(d) => match d {
          swc_css::ast::Dimension::Length(len) => {
            // let num = l.value.clone();
            // if &len.unit.value.clone() == "px" {
            //   len.unit.value = "rem".into();
            // }
          }
          swc_css::ast::Dimension::Angle(_) => todo!(),
          swc_css::ast::Dimension::Time(_) => todo!(),
          swc_css::ast::Dimension::Frequency(_) => todo!(),
          swc_css::ast::Dimension::Resolution(_) => todo!(),
          swc_css::ast::Dimension::Flex(_) => todo!(),
          swc_css::ast::Dimension::UnknownDimension(_) => todo!(),
        },
        _ => {}
      }
    }
  }

  fn visit_mut_length(&mut self, n: &mut swc_css::ast::Length) {}
}
