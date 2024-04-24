const path = require("path");

it("should work with pitching loader", () => {
	const result = require("./lib");
	expect(result).toEqual(
		path.resolve(CONTEXT, "./simple-async-loader.js") +
			"!" +
			path.resolve(CONTEXT, "./lib.js") +
			":" +
			path.resolve(CONTEXT, "./simple-loader.js") +
			"-simple"
	);
});
