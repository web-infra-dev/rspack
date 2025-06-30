use crate::{ExportsInfo, ExportsInfoData};

impl ExportsInfoData {
  pub fn set_redirect_name_to(&mut self, id: Option<ExportsInfo>) -> bool {
    if self.redirect_to() == id {
      return false;
    }
    self.set_redirect_to(id);
    true
  }
}
