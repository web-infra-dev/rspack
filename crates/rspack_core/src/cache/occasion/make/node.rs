use rspack_cacheable::SerializeError;
use rspack_cacheable::__private::rkyv::{Archive, Archived, Place, Resolver, Serialize};
use rspack_cacheable::{cacheable, Serializer};

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule,
  ModuleGraphConnection, ModuleGraphModule,
};

#[cacheable]
pub struct Node {
  pub mgm: ModuleGraphModule,
  pub module: BoxModule,
  pub dependencies: Vec<(BoxDependency, Option<AsyncDependenciesBlockIdentifier>)>,
  pub connections: Vec<ModuleGraphConnection>,
  pub blocks: Vec<AsyncDependenciesBlock>,
}

pub struct NodeRef<'a> {
  // TODO remove pub
  pub mgm: &'a ModuleGraphModule,
  pub module: &'a BoxModule,
  // TODO change to ref
  pub dependencies: Vec<(BoxDependency, Option<AsyncDependenciesBlockIdentifier>)>,
  pub connections: Vec<ModuleGraphConnection>,
  pub blocks: Vec<AsyncDependenciesBlock>,
}

impl<'a> Archive for NodeRef<'a>
where
  ModuleGraphModule: Archive,
  BoxModule: Archive,
{
  type Archived = Archived<Node>;
  type Resolver = Resolver<Node>;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    // keep the order same as Node
    let field_ptr = unsafe { &raw mut (*out.ptr()).mgm };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    <ModuleGraphModule as Archive>::resolve(self.mgm, resolver.mgm, field_out);

    let field_ptr = unsafe { &raw mut (*out.ptr()).module };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    <BoxModule as Archive>::resolve(self.module, resolver.module, field_out);

    let field_ptr = unsafe { &raw mut (*out.ptr()).dependencies };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(&self.dependencies, resolver.dependencies, field_out);

    let field_ptr = unsafe { &raw mut (*out.ptr()).connections };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(&self.connections, resolver.connections, field_out);

    let field_ptr = unsafe { &raw mut (*out.ptr()).blocks };
    let field_out = unsafe { Place::from_field_unchecked(out, field_ptr) };
    Archive::resolve(&self.blocks, resolver.blocks, field_out);
  }
}

impl<'a> Serialize<Serializer<'_>> for NodeRef<'a> {
  #[inline]
  fn serialize(
    &self,
    serializer: &mut Serializer,
  ) -> ::core::result::Result<Self::Resolver, SerializeError> {
    Ok(NodeResolver {
      mgm: Serialize::serialize(self.mgm, serializer)?,
      module: Serialize::serialize(self.module, serializer)?,
      dependencies: Serialize::serialize(&self.dependencies, serializer)?,
      connections: Serialize::serialize(&self.connections, serializer)?,
      blocks: Serialize::serialize(&self.blocks, serializer)?,
    })
  }
}
