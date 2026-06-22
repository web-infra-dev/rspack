const fs = __non_webpack_require__("fs");
const path = __non_webpack_require__("path");

// `ctx` is watched as a context dependency; its `__ignored__` subtree is
// excluded by `watchOptions.ignored`. Import two of its files directly so each
// is an individually-tracked file dependency: editing the non-ignored
// `trigger.js` drives a rebuild, while editing the ignored
// `__ignored__/seed.js` must be filtered out of the change set.
require.context("./ctx");
const trigger = require("./ctx/trigger.js");
require("./ctx/__ignored__/seed.js");

it("an edit inside an ignored subtree of a watched context is filtered from the rebuild", () => {
	if (WATCH_STEP === "0") {
		expect(trigger).toBe("initial");
		return;
	}

	if (WATCH_STEP === "1") {
		const probe = JSON.parse(
			fs.readFileSync(path.resolve(__dirname, "probe.json"), "utf-8")
		);
		const modified = probe.modifiedFiles;
		// Step 1 edited `ctx/trigger.js` and `ctx/__ignored__/seed.js` together.
		// A rebuild driven by the non-ignored edit happened (non-empty changes)...
		expect(modified.length).toBeGreaterThan(0);
		// ...while the ignored-subtree edit must be filtered from the change set.
		// (The native watcher reports the individual `ctx/__ignored__/seed.js`
		// path when unfiltered; watchpack collapses context-member changes to the
		// `ctx` directory, so this is the discriminating assertion on the native
		// path — exactly where the ignored-subtree filter lives.)
		expect(modified.some((p) => p.includes("__ignored__"))).toBe(false);
		return;
	}

	throw new Error(`unexpected watch step: ${WATCH_STEP}`);
});
