use rspack_benchmark::{Criterion, Runtime};
use rspack_tasks::within_compiler_context_for_testing_sync;

pub mod concatenate_module_code_generation;
pub mod create_chunk_assets;
pub mod create_chunk_hashes;
pub mod create_chunk_ids;
pub mod create_concatenate_module;
pub mod create_full_hash;
pub mod create_module_assets;
pub mod create_module_hashes;
pub mod create_module_ids;
pub mod create_named_chunk_ids;
pub mod create_named_module_ids;
pub mod flag_dependency_exports;
pub mod flag_dependency_usage;
pub mod mangle_exports;
pub mod real_content_hash;
pub mod runtime_requirements;
pub mod split_chunks;

pub(crate) type StageBenchmark = fn(&mut Criterion, &Runtime);

pub(crate) fn run(c: &mut Criterion, benchmark: StageBenchmark) {
  // CodSpeed simulation for this PR is dominated by runtime instrumentation noise on
  // compilation-stage microbenchmarks. Walltime and regular benchmark runs still exercise them.
  #[cfg(codspeed)]
  if rspack_benchmark::is_simulation_benchmark() {
    return;
  }

  within_compiler_context_for_testing_sync(|| {
    let rt = rspack_benchmark::build_tokio_rt();
    let _guard = rt.enter();
    benchmark(c, &rt);
  });
}
