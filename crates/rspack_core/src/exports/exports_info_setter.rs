use crate::{ExportsInfoData, RuntimeSpec, UsageState};

impl ExportsInfoData {
  pub fn set_used_for_side_effects_only(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    self.side_effects_only_info_mut().set_used_conditionally(
      Box::new(|value| value == &UsageState::Unused),
      UsageState::Used,
      runtime,
    )
  }
}
