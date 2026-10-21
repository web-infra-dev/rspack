use super::manual_adapter_model::Value as OwnedValue;

impl rspack_allocative::Allocative for OwnedValue {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut rspack_allocative::Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    let Self::Text(text) = self;
    visitor.visit_field(rspack_allocative::Key::new("manual_text"), text);
    visitor.exit();
  }
}
