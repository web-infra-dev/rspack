// #[macro_export]
// macro_rules! impl_dependency {
//   ($ident: ident, $category: ty, $type: ty) => {
//     impl rspack_core::Dependency for $ident {
//       fn id(&self) -> Option<rspack_core::DependencyId> {
//         self.id
//       }
//       fn set_id(&mut self, id: Option<rspack_core::DependencyId>) {
//         self.id = id;
//       }
//       fn category(&self) -> &rspack_core::DependencyCategory {
//         &rspack_core::$category
//       }

//       fn dependency_type(&self) -> &rspack_core::DependencyType {
//         &rspack_core::$type
//       }
//     }
//   };
// }
