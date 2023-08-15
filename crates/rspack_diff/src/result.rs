use crate::dist_diff::DistDiffResult;
#[derive(Debug)]
pub enum CliRunResult {
  DistDiffResult(DistDiffResult),
}
