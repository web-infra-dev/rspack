const fs = require("fs");
const path = require("path");

it("parse error module should have 'throw error'", () => {
	try {
		require("./recoverable.js");
		require("./non-recoverable.js");
	} catch (e) {
		expect(e.message.includes('Expected a semicolon')).toBe(true);
	}

	const output = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	let nonRecCode = /".\/non-recoverable.js":.*\n(.*)/.exec(
		output
	)[1];
	let recCode = /".\/recoverable.js":.*\n(.*)/.exec(output)[1];
	expect(nonRecCode.includes("throw new Error")).toBe(true);
	expect(recCode.includes("throw new Error")).toBe(true);
});
