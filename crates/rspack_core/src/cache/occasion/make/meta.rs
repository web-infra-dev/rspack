use rspack_cacheable::{
  cacheable, SerializeError, Serializer,
  __private::rkyv::{Archive, Archived, Place, Resolver, Serialize},
};
use rspack_collections::IdentifierSet;
use rustc_hash::FxHashSet as HashSet;

use crate::{BuildDependency, FileCounter};

#[cacheable]
pub struct Meta {
  pub make_failed_dependencies: HashSet<BuildDependency>,
  pub make_failed_module: IdentifierSet,
  pub file_dependencies: FileCounter,
  pub context_dependencies: FileCounter,
  pub missing_dependencies: FileCounter,
  pub build_dependencies: FileCounter,
  pub next_dependencies_id: u32,
}

//#[derive(
//  rspack_cacheable::__private::rkyv::Archive, rspack_cacheable::__private::rkyv::Serialize,
//)]
//#[rkyv(crate=rspack_cacheable::__private::rkyv, as=Archived<Meta>)]
pub struct MetaRef<'a> {
  pub make_failed_dependencies: &'a HashSet<BuildDependency>,
  pub make_failed_module: &'a IdentifierSet,
  pub file_dependencies: &'a FileCounter,
  pub context_dependencies: &'a FileCounter,
  pub missing_dependencies: &'a FileCounter,
  pub build_dependencies: &'a FileCounter,
  pub next_dependencies_id: u32,
}

impl<'a> Archive for MetaRef<'a> {
  type Archived = Archived<Meta>;
  type Resolver = Resolver<Meta>;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    let field_ptr = unsafe { &raw mut (*out.ptr()).make_failed_dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.make_failed_dependencies,
      resolver.make_failed_dependencies,
      field_out,
    );

    let field_ptr = unsafe { &raw mut (*out.ptr()).make_failed_module };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.make_failed_module,
      resolver.make_failed_module,
      field_out,
    );

    let field_ptr = unsafe { &raw mut (*out.ptr()).file_dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.file_dependencies,
      resolver.file_dependencies,
      field_out,
    );
    let field_ptr = unsafe { &raw mut (*out.ptr()).context_dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.context_dependencies,
      resolver.context_dependencies,
      field_out,
    );
    let field_ptr = unsafe { &raw mut (*out.ptr()).missing_dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.missing_dependencies,
      resolver.missing_dependencies,
      field_out,
    );
    let field_ptr = unsafe { &raw mut (*out.ptr()).build_dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      self.build_dependencies,
      resolver.build_dependencies,
      field_out,
    );
    let field_ptr = unsafe { &raw mut (*out.ptr()).next_dependencies_id };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(
      &self.next_dependencies_id,
      resolver.next_dependencies_id,
      field_out,
    );
  }
}

impl<'a> Serialize<Serializer<'_>> for MetaRef<'a> {
  #[inline]
  fn serialize(
    &self,
    serializer: &mut Serializer<'_>,
  ) -> ::core::result::Result<Self::Resolver, SerializeError> {
    Ok(MetaResolver {
      make_failed_dependencies: Serialize::serialize(self.make_failed_dependencies, serializer)?,
      make_failed_module: Serialize::serialize(self.make_failed_module, serializer)?,
      file_dependencies: Serialize::serialize(self.file_dependencies, serializer)?,
      context_dependencies: Serialize::serialize(self.context_dependencies, serializer)?,
      missing_dependencies: Serialize::serialize(self.missing_dependencies, serializer)?,
      build_dependencies: Serialize::serialize(self.build_dependencies, serializer)?,
      next_dependencies_id: Serialize::<Serializer>::serialize(
        &self.next_dependencies_id,
        serializer,
      )?,
    })
  }
}
