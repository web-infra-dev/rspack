use rspack_cacheable::with::AsTuple2;
use rspack_cacheable::SerializeError;
use rspack_cacheable::__private::rkyv::{
  out_field, with::With, Archive, Archived, Resolver, Serialize,
};
use rspack_cacheable::{cacheable, with::AsVec, CacheableSerializer};

use crate::{
  AsyncDependenciesBlock, AsyncDependenciesBlockIdentifier, BoxDependency, BoxModule,
  ModuleGraphConnection, ModuleGraphModule,
};

#[cacheable]
pub struct Node {
  pub mgm: ModuleGraphModule,
  pub module: BoxModule,
  #[with(AsVec<AsTuple2>)]
  pub dependencies: Vec<(BoxDependency, Option<AsyncDependenciesBlockIdentifier>)>,
  #[with(AsVec)]
  pub connections: Vec<ModuleGraphConnection>,
  #[with(AsVec)]
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

impl<'a> Archive for NodeRef<'a> {
  type Archived = Archived<Node>;
  type Resolver = Resolver<Node>;

  unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
    // keep the order same as Node
    let (fp, fo) = out_field!(out.mgm);
    Archive::resolve(self.mgm, pos + fp, resolver.mgm, fo);
    let (fp, fo) = out_field!(out.module);
    Archive::resolve(self.module, pos + fp, resolver.module, fo);
    let (fp, fo) = out_field!(out.dependencies);
    Archive::resolve(
      With::<_, AsVec<AsTuple2>>::cast(&self.dependencies),
      pos + fp,
      resolver.dependencies,
      fo,
    );
    let (fp, fo) = out_field!(out.connections);
    Archive::resolve(
      With::<_, AsVec>::cast(&self.connections),
      pos + fp,
      resolver.connections,
      fo,
    );
    let (fp, fo) = out_field!(out.blocks);
    Archive::resolve(
      With::<_, AsVec>::cast(&self.blocks),
      pos + fp,
      resolver.blocks,
      fo,
    );
  }
}

impl<'a> Serialize<CacheableSerializer> for NodeRef<'a>
//where
//  ModuleGraphModule: Serialize<CacheableSerializer>,
//  BoxModule: Serialize<CacheableSerializer>,
//  With<Vec<BoxDependency>, AsVec>: Serialize<CacheableSerializer>,
//  With<Vec<AsyncDependenciesBlock>, AsVec>: Serialize<CacheableSerializer>,
{
  #[inline]
  fn serialize(
    &self,
    serializer: &mut CacheableSerializer,
  ) -> ::core::result::Result<Self::Resolver, SerializeError> {
    Ok(NodeResolver {
      mgm: Serialize::<CacheableSerializer>::serialize(self.mgm, serializer)?,
      module: Serialize::<CacheableSerializer>::serialize(self.module, serializer)?,
      dependencies: Serialize::<CacheableSerializer>::serialize(
        With::<_, AsVec<AsTuple2>>::cast(&self.dependencies),
        serializer,
      )?,
      connections: Serialize::<CacheableSerializer>::serialize(
        With::<_, AsVec>::cast(&self.connections),
        serializer,
      )?,
      blocks: Serialize::<CacheableSerializer>::serialize(
        With::<_, AsVec>::cast(&self.blocks),
        serializer,
      )?,
    })
  }
}
