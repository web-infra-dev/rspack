use once_cell::sync::OnceCell;

use crate::utils::reference_manifest::{ClientImports, ServerReferenceManifest};

pub static SHARED_DATA: OnceCell<ServerReferenceManifest> = OnceCell::new();
pub static SHARED_CLIENT_IMPORTS: OnceCell<ClientImports> = OnceCell::new();
