const fs = require("fs");
const path = require("path");

it("build error module should have 'throw error'", () => {
	expect(() => require("./index.scss")).toThrow("SassError");
	const output = fs.readFileSync(
		path.resolve(__dirname, "bundle0.js"),
		"utf-8"
	);
	let scssCode = /throw new Error\("((?:\s|.)*?)"\)/.exec(output)[1];
	expect(scssCode).toContain("SassError");
	expect(scssCode).toContain("Module build failed");
});
