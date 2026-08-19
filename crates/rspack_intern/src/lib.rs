//! Global object interning infrastructure.
//!
//! Ported from rust-analyzer's `intern` crate (MIT OR Apache-2.0), trimmed to the non-GC mode:
//! <https://github.com/rust-lang/rust-analyzer/tree/baabc5825f3f6640e99fe32887bbeced640f825e/crates/intern>
//!
//! Equal values share one allocation, which makes equality a pointer comparison and keeps a
//! single copy of each value in memory. A value is freed once its last handle is dropped.
//!
//! [`InternedSlice`] interns a header plus a slice in one allocation, behind a thin pointer.

mod slice;

pub use self::slice::{InternSliceStorage, InternedSlice, SliceInternable};
