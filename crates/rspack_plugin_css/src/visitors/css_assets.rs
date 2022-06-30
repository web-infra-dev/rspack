use swc_css::ast::{Url, UrlValue};
use swc_css_visit::VisitMut;
use swc_atoms::JsWord;

pub struct CssAssetsComponent {
  pub transform_hook: Box<dyn Fn(String) -> String>,
}

impl VisitMut for CssAssetsComponent {
  fn visit_mut_url(&mut self, n: &mut Url) {
    let ident: String = n.name.value.to_string();
    if &ident == "url" {
      if let Some(UrlValue::Str(str)) = &mut n.value {
        let transform_hook = &self.transform_hook;
        let res: String = transform_hook(str.value.to_string());
        str.value = JsWord::from(res.clone());
        str.raw = JsWord::from(res);
      }
    }
  }
}