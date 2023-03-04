const fs = require("fs");
const path = require("path");

it("build error module should have 'throw error'", () => {
	try {
		require("./index.scss");
	} catch (e) {
		expect(e.message.includes('Undefined variable.')).toBe(true);
	}
		
	const output = fs.readFileSync(path.resolve(__dirname, "main.js"), "utf-8");
	let scssCode = /".\/index.scss":.*\n(.*)/.exec(
		output
	)[1];
	expect(scssCode.includes("throw new Error")).toBe(true);
});
