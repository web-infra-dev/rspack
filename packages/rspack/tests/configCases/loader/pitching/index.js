const path = require("path");

it("should work with pitching loader", () => {
	const result = require("./lib");
	expect(result).toEqual(
		path.resolve(__dirname, "../simple-async-loader.js") +
			"!" +
			path.resolve(__dirname, "../lib.js") +
			":" +
			path.resolve(__dirname, "../simple-loader.js") +
			"-simple"
	);
});
