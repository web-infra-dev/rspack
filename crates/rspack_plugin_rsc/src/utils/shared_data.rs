use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::utils::decl::{ClientImports, ServerReferenceManifest};

pub static SHARED_DATA: Lazy<Mutex<ServerReferenceManifest>> = Lazy::new(|| Mutex::default());
// Collected client imports, group by entry name or route chunk name
pub static SHARED_CLIENT_IMPORTS: Lazy<Mutex<ClientImports>> = Lazy::new(|| Mutex::default());
pub static SHARED_SERVER_IMPORTS: Lazy<Mutex<ClientImports>> = Lazy::new(|| Mutex::default());
