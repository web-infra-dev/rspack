const path = require("path");

it("should work with pitching loader", () => {
	const result = require("./lib");
	let exports = result
		.replaceAll("builtin:test-simple-async-loader", path.resolve(CONTEXT, "./simple-async-loader.js"))
		.replaceAll("builtin:test-simple-loader", path.resolve(CONTEXT, "./simple-loader.js"))
		.replaceAll("builtin:test-pitching-loader", path.resolve(CONTEXT, "./pitching-loader.js"))
	expect(exports).toEqual(
		path.resolve(CONTEXT, "./simple-async-loader.js") +
			"!" +
			path.resolve(CONTEXT, "./lib.js") +
			":" +
			path.resolve(CONTEXT, "./simple-loader.js") +
			"-simple"
	);
});
