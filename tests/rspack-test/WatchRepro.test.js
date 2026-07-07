// [WATCH-DIAG] TEMPORARY: reproduce the WASM-only watch flaky of
// `side-effects/issue-7400 > should compile step 1`.
//
// Symptom: at step 1 the executed bundle still contains step 0's
// `expect(WATCH_STEP).toEqual("0")` while WATCH_STEP is already "1" -> the
// runner executes a stale (previous-step) bundle.
//
// Findings so far (instrumented CI runs): the harness step-sync is robust -
// every case emits exactly 2 Build events (one per step), always with correct
// content (STALE=false), even under event-loop freeze. So the leaked-Build
// hypothesis is falsified. This iteration bumps sample volume (N) and uses a
// NON-UNIFORM freeze to create differential timer misalignment instead of a
// uniform time shift.
//
// Only runs under WASM; native jobs register nothing.
if (process.env.WASM) {
	process.env.WATCH_DIAG = "1";

	const path = require("path");
	const { createWatchCase } = require("@rspack/test-tools");

	// Non-uniform event-loop freezes: cycle through varied block durations so
	// watchpack poll/aggregate timers and the step-sync setTimeout drift apart.
	if (process.env.WATCH_REPRO_FREEZE !== "0") {
		const blocks = [150, 600, 350, 800, 250, 500];
		let k = 0;
		const tick = () => {
			const end = Date.now() + blocks[k++ % blocks.length];
			while (Date.now() < end) {}
			const t = setTimeout(tick, 180);
			if (t.unref) t.unref();
		};
		const t0 = setTimeout(tick, 180);
		if (t0.unref) t0.unref();
	}

	const src = path.join(__dirname, "watchCases", "side-effects", "issue-7400");
	const count = Number(process.env.WATCH_REPRO_N || 500);

	for (let i = 0; i < count; i++) {
		const name = `side-effects/issue-7400-repro-${String(i).padStart(4, "0")}`;
		const dist = path.resolve(__dirname, "./js/watch-repro", name);
		const temp = path.resolve(__dirname, "./js/temp-repro", name);
		createWatchCase(name, src, dist, temp);
	}
}
