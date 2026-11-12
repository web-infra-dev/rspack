//! Traverse the public PnP manifest. Its private trie and regex remain boundaries.
use crate::{Allocative, Key, Visitor};
macro_rules! fields {
 ($ty:ty, $($field:ident),* $(,)?) => {
  impl Allocative for $ty {
   fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    $(visitor.visit_field(Key::new(stringify!($field)), &self.$field);)*
    visitor.exit();
   }
  }
 };
}
fields!(pnp::PackageLocator, name, reference);
fields!(
  pnp::PackageInformation,
  package_location,
  package_dependencies
);
impl Allocative for pnp::PackageDependency {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    match self {
      Self::Reference(value) => visitor.visit_field(Key::new("reference"), value),
      Self::Alias(name, value) => {
        visitor.visit_field(Key::new("name"), name);
        visitor.visit_field(Key::new("reference"), value);
      }
    }
    visitor.exit();
  }
}
impl Allocative for pnp::Manifest {
  fn visit<'a, 'b: 'a>(&self, visitor: &'a mut Visitor<'b>) {
    let mut visitor = visitor.enter_self(self);
    visitor.visit_field(Key::new("manifest_dir"), &self.manifest_dir);
    visitor.visit_field(Key::new("manifest_path"), &self.manifest_path);
    visitor.visit_field(
      Key::new("dependency_tree_roots"),
      &self.dependency_tree_roots,
    );
    visitor.visit_field(Key::new("fallback_pool"), &self.fallback_pool);
    visitor.visit_field(
      Key::new("fallback_exclusion_list"),
      &self.fallback_exclusion_list,
    );
    visitor.visit_field(
      Key::new("package_registry_data"),
      &self.package_registry_data,
    );
    visitor.visit_opaque(&self.location_trie);
    if let Some(regex) = &self.ignore_pattern_data {
      visitor.visit_opaque(regex);
    }
    visitor.exit();
  }
}
