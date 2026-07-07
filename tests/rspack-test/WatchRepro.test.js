// [WATCH-DIAG] TEMPORARY: reproduce the WASM-only watch flaky of
// `side-effects/issue-7400 > should compile step 1`.
//
// Hypothesis: the step-sync in createWatchStepProcessor uses `once(Build)` +
// a fixed 100ms sleep. Under slow WASM compilation a trailing Build event from
// the previous step can resolve the next step's task before its own copyDiff,
// so the runner executes a stale (previous-step) bundle -> WATCH_STEP="1" but
// the bundle asserts toEqual("0"). This file registers many identical copies of
// the case (distinct temp/dist) to amplify the hit rate, with WATCH_DIAG on.
//
// Only runs under WASM; native jobs register nothing.
if (process.env.WASM) {
	process.env.WATCH_DIAG = "1";

	const path = require("path");
	const { createWatchCase } = require("@rspack/test-tools");

	const src = path.join(
		__dirname,
		"watchCases",
		"side-effects",
		"issue-7400"
	);
	const count = Number(process.env.WATCH_REPRO_N || 50);

	for (let i = 0; i < count; i++) {
		const name = `side-effects/issue-7400-repro-${String(i).padStart(3, "0")}`;
		const dist = path.resolve(__dirname, "./js/watch-repro", name);
		const temp = path.resolve(__dirname, "./js/temp-repro", name);
		createWatchCase(name, src, dist, temp);
	}
}
