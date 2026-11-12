const path = require("path");

it("should work with pitching loader", () => {
	const result = require("./lib");
	let exports = result
		.replaceAll("builtin:test-simple-async-loader", path.resolve(CONTEXT, "./simple-async-loader.mjs"))
		.replaceAll("builtin:test-simple-loader", path.resolve(CONTEXT, "./simple-loader.mjs"))
		.replaceAll("builtin:test-pitching-loader", path.resolve(CONTEXT, "./pitching-loader.mjs"))
		.replaceAll(/\?\?ruleSet\[\d\]\.rules\[\d\]\.use\[\d\]/g, "");

	expect(exports).toEqual(
		path.resolve(CONTEXT, "./simple-async-loader.mjs") +
		"!" +
		path.resolve(CONTEXT, "./lib.js") +
		":" +
		path.resolve(CONTEXT, "./simple-loader.mjs") +
		"-simple"
	);
});
