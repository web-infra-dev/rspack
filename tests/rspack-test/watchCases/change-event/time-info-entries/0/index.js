const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

// Register a context dependency so contextTimestamps is non-empty on rebuild.
require.context("./ctx").keys();

it("native watcher populates file/context time info entries", () => {
	if (WATCH_STEP === "0") {
		// The initial build does not populate timestamps; they are set on the
		// first watcher callback, which fires before the step-1 bundle runs.
		return;
	}

	const probe = JSON.parse(
		fs.readFileSync(path.resolve(__dirname, "probe.json"), "utf-8")
	);

	// fileTimestamps populated; the entry file has numeric safeTime + timestamp.
	expect(Array.isArray(probe.file)).toBe(true);
	expect(probe.file.length).toBeGreaterThan(0);
	const entry = probe.file.find(([p]) => p.endsWith("index.js"));
	expect(entry).toBeTruthy();
	expect(typeof entry[1].safeTime).toBe("number");
	expect(entry[1].safeTime).toBeGreaterThan(0);
	expect(typeof entry[1].timestamp).toBe("number");

	// contextTimestamps populated; the ctx directory has a numeric safeTime.
	expect(Array.isArray(probe.context)).toBe(true);
	expect(probe.context.length).toBeGreaterThan(0);
	const ctx = probe.context.find(([p]) => p.endsWith("ctx"));
	expect(ctx).toBeTruthy();
	expect(typeof ctx[1].safeTime).toBe("number");
	expect(ctx[1].safeTime).toBeGreaterThan(0);
});
