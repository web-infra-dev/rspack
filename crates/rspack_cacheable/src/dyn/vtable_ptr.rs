use rkyv::ptr_meta::DynMetadata;

/// A untyped wrapper for vtable
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct VTablePtr(DynMetadata<()>);

impl VTablePtr {
  pub const unsafe fn new<T: ?Sized>(vtable: DynMetadata<T>) -> Self {
    Self(core::mem::transmute(vtable))
  }
  pub const unsafe fn cast<T: ?Sized>(self) -> DynMetadata<T> {
    core::mem::transmute(self.0)
  }
}
unsafe impl Send for VTablePtr {}
unsafe impl Sync for VTablePtr {}
