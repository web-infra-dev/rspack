
use smol_str::SmolStr;


#[derive(Debug)]
pub struct JsExtModule {
    pub id: SmolStr,
}

impl JsExtModule {
    pub fn new(id: SmolStr) -> Self {
        Self {
            id,
            // resolved_ids: Default::default(),
            // source: "".to_string(),
        }
    }
}
