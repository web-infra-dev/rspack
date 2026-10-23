//! Shared filesystem identities and immutable snapshot data.
//!
//! Each FileTable is one filesystem namespace. Runtime observations and snapshot
//! states are introduced in subsequent steps described in DESIGN.md.

mod file_table;

pub(crate) use file_table::{FileId, FileRef, FileTable, FileTableStatistics};
