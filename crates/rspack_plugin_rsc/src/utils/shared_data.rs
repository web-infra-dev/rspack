use once_cell::sync::OnceCell;

use crate::utils::reference_manifest::ServerReferenceManifest;

pub static SHARED_DATA: OnceCell<ServerReferenceManifest> = OnceCell::new();
