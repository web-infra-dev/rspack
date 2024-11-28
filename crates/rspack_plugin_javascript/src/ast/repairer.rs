use swc_core::{
  common::{collections::AHashMap as HashMap, Span},
  ecma::visit::VisitMut,
};

pub(crate) struct AstSpanRepairer {
  pub(crate) span_map: HashMap<Span, Span>,
}

impl VisitMut for AstSpanRepairer {
  fn visit_mut_span(&mut self, span: &mut Span) {
    if let Some(new_span) = self.span_map.get(span) {
      *span = *new_span;
    }
  }
}
