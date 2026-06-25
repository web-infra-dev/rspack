const fs = require("fs");
const path = require("path");
const value = require("./trigger");

it("a change inside an ignored subtree must not pre-empt the real rebuild", () => {
	if (WATCH_STEP === "0") {
		expect(value).toBe("initial");
	} else if (WATCH_STEP === "1") {
		expect(value).toBe("changed");
		const probe = JSON.parse(
			fs.readFileSync(path.resolve(__dirname, "probe.json"), "utf-8")
		);
		// Exactly two builds: the initial one and the `trigger.js` change. If the
		// ignored change had rebuilt, this would be greater than 2.
		expect(probe.builds).toBe(2);
	} else {
		throw new Error(`unexpected watch step: ${WATCH_STEP}`);
	}
});
