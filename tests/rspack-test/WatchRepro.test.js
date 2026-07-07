// [WATCH-DIAG] TEMPORARY: reproduce the WASM-only watch flaky of
// `side-effects/issue-7400 > should compile step 1`.
//
// Symptom: at step 1 the executed bundle still contains step 0's
// `expect(WATCH_STEP).toEqual("0")` while WATCH_STEP is already "1" -> the
// runner executes a stale (previous-step) bundle.
//
// Hypothesis: createWatchStepProcessor synchronizes step transitions with
// `once(Build)` + a fixed 100ms sleep. Under event-loop pressure a Build event
// that does NOT correspond to this step's copyDiff resolves the step's task, so
// the runner runs a stale bundle. Iteration 1 (no pressure) showed a clean
// ~1200ms single-Build path with STALE=false, so we now inject event-loop
// freezes (the documented behavior of heavy sibling WASM tests) to recreate the
// real contention.
//
// Only runs under WASM; native jobs register nothing.
if (process.env.WASM) {
	process.env.WATCH_DIAG = "1";

	const path = require("path");
	const { createWatchCase } = require("@rspack/test-tools");

	// Simulate the ~1s event-loop freezes that heavy sibling WASM tests cause.
	if (process.env.WATCH_REPRO_FREEZE !== "0") {
		const blockMs = Number(process.env.WATCH_REPRO_FREEZE_MS || 400);
		const gapMs = Number(process.env.WATCH_REPRO_FREEZE_GAP || 500);
		const timer = setInterval(() => {
			const end = Date.now() + blockMs;
			while (Date.now() < end) {}
		}, blockMs + gapMs);
		if (timer.unref) timer.unref();
	}

	const src = path.join(__dirname, "watchCases", "side-effects", "issue-7400");
	const count = Number(process.env.WATCH_REPRO_N || 100);

	for (let i = 0; i < count; i++) {
		const name = `side-effects/issue-7400-repro-${String(i).padStart(3, "0")}`;
		const dist = path.resolve(__dirname, "./js/watch-repro", name);
		const temp = path.resolve(__dirname, "./js/temp-repro", name);
		createWatchCase(name, src, dist, temp);
	}
}
