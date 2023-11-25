const fs = require("fs");
const path = require("path");

it("generated code should use hotUpdateGlobal", async () => {
	const a = await fs.promises.readFile(
		path.resolve(__dirname, "a.js"),
		"utf-8"
	);
	const b = await fs.promises.readFile(
		path.resolve(__dirname, "b.js"),
		"utf-8"
	);
	expect(a).toContain('self["webpackHotUpdatea"]');
	expect(b).toContain('self["webpackHotUpdateb"]');
});
