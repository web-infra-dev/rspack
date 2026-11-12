const fs = require("fs");
const path = require("path");
const value = require("./trigger");
// A dependency the `ignored` function rejects: it is built, but never watched.
require("./__ignored__/dep");

it("a change to a dependency the `ignored` function rejects must not rebuild", () => {
	if (WATCH_STEP === "0") {
		expect(value).toBe("initial");
	} else if (WATCH_STEP === "1") {
		expect(value).toBe("changed");
		const probe = JSON.parse(
			fs.readFileSync(path.resolve(__dirname, "probe.json"), "utf-8")
		);
		// Exactly two builds: the initial one and the `trigger.js` change. If the
		// ignored dependency had rebuilt, this would be greater than 2.
		expect(probe.builds).toBe(2);
	} else {
		throw new Error(`unexpected watch step: ${WATCH_STEP}`);
	}
});
