case_entry!(|c| {
  if crate::groups::bundle::threejs_10x::enabled() {
    crate::groups::bundle::bundle_benchmark_case(c, "threejs-10x-production-sourcemap");
  }
});
