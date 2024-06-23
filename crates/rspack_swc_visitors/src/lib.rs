mod react;
pub use react::ReactOptions;

mod preact;
pub use preact::{preact, PreactOptions, RawPreactOptions};

mod define;
pub use define::{define, Define, RawDefine};

pub type Provide = std::collections::HashMap<String, Vec<String>>;

mod import;
pub use import::{import, CustomTransform, ImportOptions, RawImportOptions, StyleConfig};
