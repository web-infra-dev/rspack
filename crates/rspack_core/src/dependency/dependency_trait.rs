use std::{
  alloc::{Layout, LayoutError, alloc, handle_alloc_error},
  any::Any,
  fmt::{self, Debug},
  ops::{Deref, DerefMut},
  ptr,
  sync::{Arc, OnceLock, atomic::AtomicUsize},
};

use rspack_cacheable::{
  cacheable_dyn,
  rkyv::{
    Archive, ArchiveUnsized, Deserialize, DeserializeUnsized, Place, Serialize, SerializeUnsized,
    de::{FromMetadata, Metadata, Pooling, PoolingExt, SharedPointer},
    ptr_meta::{Pointee, from_raw_parts_mut},
    rancor::{Fallible, ResultExt, Source},
    rc::{ArchivedRc, Flavor, RcResolver},
    ser::{Sharing, Writer},
    traits::LayoutRaw,
  },
};
use rspack_collections::{IdentifierMap, IdentifierSet};
use rspack_error::Diagnostic;
use rspack_location::DependencyLocation;
use rspack_util::ext::AsAny;
use triomphe::{Arc as TriompheArc, UniqueArc};
use unsize::{CoerceUnsize, Coercion};

use super::{
  DependencyCategory, DependencyId, DependencyRange, DependencyType, ExportsSpec,
  dependency_template::AsDependencyCodeGeneration, module_dependency::*,
};
use crate::{
  AsContextDependency, ConnectionState, Context, ExportsInfoArtifact, ForwardId, ImportAttributes,
  ImportPhase, JavascriptParserUrl, LazyUntil, Module, ModuleGraph, ModuleGraphCacheArtifact,
  ModuleLayer, ReferencedExport, RuntimeSpec, SideEffectsStateArtifact,
  create_exports_object_referenced,
};

#[derive(Debug, Clone, Copy)]
pub enum AffectType {
  True,
  False,
  Transitive,
}

/// Module-scoped state shared while collecting diagnostics from its dependencies.
#[derive(Debug, Default)]
pub struct DependencyDiagnosticsContext {
  module_source: OnceLock<Option<Arc<str>>>,
}

impl DependencyDiagnosticsContext {
  fn get_or_init_module_source(&self, init: impl FnOnce() -> Option<Arc<str>>) -> Option<Arc<str>> {
    self.module_source.get_or_init(init).clone()
  }

  /// Lazily materialize the module source once and share it across its diagnostics.
  pub fn module_source(&self, module: &dyn Module) -> Option<Arc<str>> {
    self.get_or_init_module_source(|| {
      module
        .source()
        .map(|source| source.source().into_string_lossy().into())
    })
  }
}

#[cacheable_dyn]
pub trait Dependency:
  AsDependencyCodeGeneration + AsContextDependency + AsModuleDependency + AsAny + Send + Sync + Debug
{
  fn id(&self) -> &DependencyId;

  fn category(&self) -> &DependencyCategory {
    &DependencyCategory::Unknown
  }

  fn dependency_type(&self) -> &DependencyType {
    &DependencyType::Unknown
  }

  /// Whether this dependency should be excluded when a global entry include is applied to an
  /// async entrypoint.
  fn skip_async_entrypoints(&self) -> bool {
    false
  }

  fn url_mode(&self) -> Option<JavascriptParserUrl> {
    None
  }

  // get issuer context
  fn get_context(&self) -> Option<&Context> {
    None
  }

  // get issuer layer
  fn get_layer(&self) -> Option<&ModuleLayer> {
    None
  }

  fn get_phase(&self) -> ImportPhase {
    ImportPhase::Evaluation
  }

  fn get_attributes(&self) -> Option<&ImportAttributes> {
    None
  }

  fn get_exports(
    &self,
    _mg: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<ExportsSpec> {
    None
  }

  fn get_module_evaluation_side_effects_state(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _side_effects_state_artifact: &SideEffectsStateArtifact,
    _module_chain: &mut IdentifierSet,
    _connection_state_cache: &mut IdentifierMap<ConnectionState>,
  ) -> ConnectionState {
    ConnectionState::Active(true)
  }

  fn loc(&self) -> Option<DependencyLocation> {
    None
  }

  fn range(&self) -> Option<DependencyRange> {
    None
  }

  fn source_order(&self) -> Option<i32> {
    None
  }

  fn resource_identifier(&self) -> Option<&str> {
    None
  }

  fn get_diagnostics(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
  ) -> Option<Vec<Diagnostic>> {
    None
  }

  fn get_diagnostics_with_context(
    &self,
    module_graph: &ModuleGraph,
    module_graph_cache: &ModuleGraphCacheArtifact,
    exports_info_artifact: &ExportsInfoArtifact,
    _context: &DependencyDiagnosticsContext,
  ) -> Option<Vec<Diagnostic>> {
    self.get_diagnostics(module_graph, module_graph_cache, exports_info_artifact)
  }

  fn get_referenced_exports(
    &self,
    _module_graph: &ModuleGraph,
    _module_graph_cache: &ModuleGraphCacheArtifact,
    _exports_info_artifact: &ExportsInfoArtifact,
    _runtime: Option<&RuntimeSpec>,
  ) -> Vec<ReferencedExport> {
    create_exports_object_referenced()
  }

  fn could_affect_referencing_module(&self) -> AffectType;

  fn forward_id(&self) -> ForwardId {
    ForwardId::All
  }

  fn lazy(&self) -> Option<LazyUntil> {
    None
  }

  fn set_lazy(&self) {}

  fn unset_lazy(&self) -> bool {
    false
  }
}

impl dyn Dependency + '_ {
  pub fn downcast_ref<D: Any>(&self) -> Option<&D> {
    self.as_any().downcast_ref::<D>()
  }

  pub fn downcast_mut<D: Any>(&mut self) -> Option<&mut D> {
    self.as_any_mut().downcast_mut::<D>()
  }

  pub fn is<D: Any>(&self) -> bool {
    self.downcast_ref::<D>().is_some()
  }
}

/// A dependency with unique ownership while it is being constructed.
///
/// Unlike `Box<dyn Dependency>`, this uses the same allocation layout as [`DependencyRef`], so
/// publishing it into the module graph does not reallocate or move the dependency.
pub struct UniqueDependency(UniqueArc<dyn Dependency>);

impl UniqueDependency {
  pub fn new<D: Dependency + 'static>(dependency: D) -> Self {
    let coercion =
      unsafe { Coercion::<D, dyn Dependency>::new(|ptr: *const D| ptr as *const dyn Dependency) };
    Self(UniqueArc::new(dependency).unsize(coercion))
  }

  pub fn shareable(self) -> DependencyRef {
    DependencyRef(self.0.shareable())
  }
}

impl Debug for UniqueDependency {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(self.as_ref(), f)
  }
}

impl Deref for UniqueDependency {
  type Target = dyn Dependency;

  fn deref(&self) -> &Self::Target {
    &*self.0
  }
}

impl DerefMut for UniqueDependency {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut *self.0
  }
}

impl AsRef<dyn Dependency> for UniqueDependency {
  fn as_ref(&self) -> &(dyn Dependency + 'static) {
    &*self.0
  }
}

impl AsMut<dyn Dependency> for UniqueDependency {
  fn as_mut(&mut self) -> &mut (dyn Dependency + 'static) {
    &mut *self.0
  }
}

impl Archive for UniqueDependency {
  type Archived = ArchivedRc<<dyn Dependency as ArchiveUnsized>::Archived, DependencyRefFlavor>;
  type Resolver = RcResolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedRc::resolve_from_ref(self.as_ref(), resolver, out);
  }
}

impl<S> Serialize<S> for UniqueDependency
where
  dyn Dependency: SerializeUnsized<S>,
  S: Writer + Sharing + Fallible + ?Sized,
  S::Error: Source,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedRc::<
      <dyn Dependency as ArchiveUnsized>::Archived,
      DependencyRefFlavor,
    >::serialize_from_ref(self.as_ref(), serializer)
  }
}

impl<D> Deserialize<UniqueDependency, D>
  for ArchivedRc<<dyn Dependency as ArchiveUnsized>::Archived, DependencyRefFlavor>
where
  <dyn Dependency as Pointee>::Metadata: Into<Metadata> + FromMetadata,
  <dyn Dependency as ArchiveUnsized>::Archived: DeserializeUnsized<dyn Dependency, D>,
  D: Fallible + ?Sized,
  D::Error: Source,
{
  fn deserialize(&self, deserializer: &mut D) -> Result<UniqueDependency, D::Error> {
    let metadata = self.get().deserialize_metadata();
    let out = <DependencyRef as SharedPointer<dyn Dependency>>::alloc(metadata).into_error()?;
    unsafe {
      self.get().deserialize_unsized(deserializer, out)?;
    }
    let raw = unsafe { <DependencyRef as SharedPointer<dyn Dependency>>::from_value(out) };
    let arc = unsafe { TriompheArc::from_raw(raw) };
    let unique = UniqueArc::try_from(arc)
      .unwrap_or_else(|_| unreachable!("a freshly deserialized dependency has one owner"));
    Ok(UniqueDependency(unique))
  }
}

/// Compatibility name for dependency construction sites. The backing allocation is a
/// [`UniqueArc`], not a `Box`.
pub type BoxDependency = UniqueDependency;

/// A shared dependency published into the module graph.
///
/// This newtype also supplies rkyv with the dynamically sized allocation support that
/// `triomphe::Arc` does not currently expose for trait objects.
pub struct DependencyRef(TriompheArc<dyn Dependency>);

impl DependencyRef {
  pub fn new<D: Dependency + 'static>(dependency: D) -> Self {
    UniqueDependency::new(dependency).shareable()
  }
}

impl From<UniqueDependency> for DependencyRef {
  fn from(dependency: UniqueDependency) -> Self {
    dependency.shareable()
  }
}

impl Clone for DependencyRef {
  fn clone(&self) -> Self {
    Self(self.0.clone())
  }
}

impl Debug for DependencyRef {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    Debug::fmt(self.as_ref(), f)
  }
}

impl Deref for DependencyRef {
  type Target = dyn Dependency;

  fn deref(&self) -> &Self::Target {
    &*self.0
  }
}

impl AsRef<dyn Dependency> for DependencyRef {
  fn as_ref(&self) -> &(dyn Dependency + 'static) {
    &*self.0
  }
}

pub struct DependencyRefFlavor;

impl Flavor for DependencyRefFlavor {
  const ALLOW_CYCLES: bool = false;
}

unsafe impl SharedPointer<dyn Dependency> for DependencyRef {
  fn alloc(
    metadata: <dyn Dependency as Pointee>::Metadata,
  ) -> Result<*mut dyn Dependency, LayoutError> {
    let value_layout = <dyn Dependency as LayoutRaw>::layout_raw(metadata)?;
    let (layout, data_offset) = Layout::new::<AtomicUsize>().extend(value_layout)?;
    let layout = layout.pad_to_align();
    let allocation = unsafe { alloc(layout) };
    if allocation.is_null() {
      handle_alloc_error(layout);
    }

    unsafe {
      ptr::write(allocation.cast::<AtomicUsize>(), AtomicUsize::new(1));
      Ok(from_raw_parts_mut(
        allocation.add(data_offset).cast(),
        metadata,
      ))
    }
  }

  unsafe fn from_value(ptr: *mut dyn Dependency) -> *mut dyn Dependency {
    ptr
  }

  unsafe fn drop(ptr: *mut dyn Dependency) {
    drop(unsafe { TriompheArc::from_raw(ptr) });
  }
}

impl Archive for DependencyRef {
  type Archived = ArchivedRc<<dyn Dependency as ArchiveUnsized>::Archived, DependencyRefFlavor>;
  type Resolver = RcResolver;

  fn resolve(&self, resolver: Self::Resolver, out: Place<Self::Archived>) {
    ArchivedRc::resolve_from_ref(self.as_ref(), resolver, out);
  }
}

impl<S> Serialize<S> for DependencyRef
where
  dyn Dependency: SerializeUnsized<S>,
  S: Writer + Sharing + Fallible + ?Sized,
  S::Error: Source,
{
  fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
    ArchivedRc::<
      <dyn Dependency as ArchiveUnsized>::Archived,
      DependencyRefFlavor,
    >::serialize_from_ref(self.as_ref(), serializer)
  }
}

impl<D> Deserialize<DependencyRef, D>
  for ArchivedRc<<dyn Dependency as ArchiveUnsized>::Archived, DependencyRefFlavor>
where
  <dyn Dependency as Pointee>::Metadata: Into<Metadata> + FromMetadata,
  <dyn Dependency as ArchiveUnsized>::Archived: DeserializeUnsized<dyn Dependency, D>,
  D: Pooling + Fallible + ?Sized,
  D::Error: Source,
{
  fn deserialize(&self, deserializer: &mut D) -> Result<DependencyRef, D::Error> {
    let raw = deserializer.deserialize_shared::<_, DependencyRef>(self.get())?;
    let arc = unsafe { TriompheArc::from_raw(raw) };
    let _ = TriompheArc::into_raw(arc.clone());
    Ok(DependencyRef(arc))
  }
}
