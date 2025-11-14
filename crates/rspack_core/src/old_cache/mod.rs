use std::{path::PathBuf, sync::Arc};

use crate::CompilerOptions;

mod occasion;
mod storage;
use occasion::{CodeGenerateOccasion, ProcessRuntimeRequirementsOccasion};
use storage::new_storage;

#[derive(Debug)]
pub struct Cache {
  pub code_generate_occasion: CodeGenerateOccasion,
  pub process_runtime_requirements_occasion: ProcessRuntimeRequirementsOccasion,
}

impl Cache {
  pub fn new(options: Arc<CompilerOptions>) -> Self {
    Self {
      code_generate_occasion: CodeGenerateOccasion::new(new_storage(&options.cache)),
      process_runtime_requirements_occasion: ProcessRuntimeRequirementsOccasion::new(new_storage(
        &options.cache,
      )),
    }
  }

  pub fn set_modified_files(&self, _modified_files: Vec<PathBuf>) {
    // TODO
  }

  pub fn begin_idle(&self) {
    self.code_generate_occasion.begin_idle();
    self.process_runtime_requirements_occasion.begin_idle();
  }

  pub fn end_idle(&self) {}
}
