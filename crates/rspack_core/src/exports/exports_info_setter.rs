use crate::{ExportsInfo, ExportsInfoData, RuntimeSpec, UsageState};

impl ExportsInfoData {
  pub fn set_redirect_name_to(&mut self, id: Option<ExportsInfo>) -> bool {
    if self.redirect_to() == id {
      return false;
    }
    self.set_redirect_to(id);
    true
  }

  pub fn set_used_for_side_effects_only(&mut self, runtime: Option<&RuntimeSpec>) -> bool {
    self.side_effects_only_info_mut().set_used_conditionally(
      Box::new(|value| value == &UsageState::Unused),
      UsageState::Used,
      runtime,
    )
  }
}
