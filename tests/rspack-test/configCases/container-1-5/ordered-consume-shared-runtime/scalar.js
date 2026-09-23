const fs = require("fs");
const path = require("path");

// This entry only has a scalar eager consume, but it shares a runtime with
// `ordered.js`. The runtime installs all initial consumes behind the ordered
// scope's initialization barrier, so this entry must await it as well.
const scalarLib = require("scalar-lib");

it("should resolve the scalar eager consume from an entry sharing an async runtime", () => {
	expect(scalarLib).toBe("scalar-lib");
});

it("should give every entry the startup mode of its runtime", () => {
	// An entry that delegates to a shared runtime gets the synchronous
	// federation startup call prepended when its startup mode is decided per
	// chunk, even though the runtime it shares with `ordered.js` is
	// asynchronous. Every entry must follow its runtime's plan. (The pattern is
	// built without the literal: this test's source is part of the bundle.)
	const syncStartupCall = new RegExp(["Federation ", "startup call"].join(""));
	const hasSyncStartupCall = (name) =>
		syncStartupCall.test(fs.readFileSync(path.join(__dirname, name), "utf-8"));
	expect(hasSyncStartupCall("scalar.js")).toBe(false);
	expect(hasSyncStartupCall("ordered.js")).toBe(false);
});
