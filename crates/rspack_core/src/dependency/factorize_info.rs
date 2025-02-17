use rspack_error::Diagnostic;
use rspack_paths::ArcPath;
use rustc_hash::FxHashSet as HashSet;

pub struct FactorizeInfo {
  file_dependencies: HashSet<ArcPath>,
  context_dependencies: HashSet<ArcPath>,
  missing_dependencies: HashSet<ArcPath>,
  diagnostics: Vec<Diagnostic>,
}

/*impl FactorizeInfo {
    pub fn depends_on(&self, modified_file: &HashSet<ArcPath>) -> bool {
        for item in modified_file {
            if self.file_dependencies.contains(item)
                || build_info.build_dependencies.contains(item)
                || build_info.context_dependencies.contains(item)
                || build_info.missing_dependencies.contains(item)
            {
                return true;
            }
        }

        false
    }
}
*/
