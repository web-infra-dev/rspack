use crate::dist_diff::command::DistDiffResult;
#[derive(Debug)]
pub enum CliRunResult {
  DistDiffResult(DistDiffResult),
}
