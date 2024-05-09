use std::sync::Mutex;

use once_cell::sync::Lazy;

use crate::utils::reference_manifest::{ClientImports, ServerReferenceManifest};

pub static SHARED_DATA: Lazy<Mutex<ServerReferenceManifest>> = Lazy::new(|| Mutex::default());
pub static SHARED_CLIENT_IMPORTS: Lazy<Mutex<ClientImports>> = Lazy::new(|| Mutex::default());
