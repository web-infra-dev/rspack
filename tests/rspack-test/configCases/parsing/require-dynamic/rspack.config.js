/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle0: "./index.js",
		test: "./test.js"
	},
	module: {
		parser: {
			javascript: {
				requireDynamic: false,
				// To preserve `require(...)`, we need to use `requireAsExpression: false` alongside.
				requireAsExpression: false
			}
		}
	},
	output: {
		filename: "[name].js"
	},
	node: {
		__dirname: false
	}
};
