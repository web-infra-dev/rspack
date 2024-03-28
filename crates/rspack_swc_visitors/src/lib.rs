mod react;
pub use react::ReactOptions;

mod preact;
pub use preact::{preact, PreactOptions, RawPreactOptions};

mod define;
pub use define::{define, Define, RawDefine};

pub type Provide = std::collections::HashMap<String, Vec<String>>;

mod relay;
pub use relay::{relay, RawRelayOptions, RelayLanguageConfig, RelayOptions};

mod import;
pub use import::{import, CustomTransform, ImportOptions, RawImportOptions, StyleConfig};

mod emotion;
pub use emotion::{emotion, EmotionOptions, RawEmotionOptions};

mod styled_components;
pub use crate::styled_components::{
  styled_components, RawStyledComponentsOptions, StyledComponentsOptions,
};
