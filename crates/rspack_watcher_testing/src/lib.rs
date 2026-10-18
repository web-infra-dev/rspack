#![cfg(test)]

// Compile the implementation here so lifecycle tests need no public test API.
include!("../../rspack_watcher/src/lib.rs");

mod ancestor_events;
mod incremental;
mod lifecycle;
mod rescan;
mod scan_requests;
mod subscriptions;
mod type_changes;

mod time_info;

mod directory_analyzer;

mod root_analyzer;

#[path = "polling.rs"]
mod polling_tests;

mod native_lifecycle;
