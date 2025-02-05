use rkyv::ptr_meta::DynMetadata;

/// A untyped wrapper for vtable
#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub struct VTablePtr(DynMetadata<()>);

impl VTablePtr {
  pub const fn new<T: ?Sized>(vtable: DynMetadata<T>) -> Self {
    // Creating VTablePtr is safe while using it is unsafe, just like raw pointers in rust.
    Self(unsafe { core::mem::transmute::<DynMetadata<T>, DynMetadata<()>>(vtable) })
  }

  /// # Safety
  /// The casting target `T` must be consistent with the `T` in VTablePtr::new<T>
  /// Currently it is implemented by store VTablePtr as values in HashMap to associate the types with __DYN_ID
  pub const unsafe fn cast<T: ?Sized>(self) -> DynMetadata<T> {
    core::mem::transmute(self.0)
  }
}
unsafe impl Send for VTablePtr {}
unsafe impl Sync for VTablePtr {}
