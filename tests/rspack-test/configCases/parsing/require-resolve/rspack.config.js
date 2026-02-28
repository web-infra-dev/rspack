/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		bundle0: "./index.js",
		test: "./test.js"
	},
	module: {
		parser: {
			javascript: {
				requireResolve: false,
				// To preserve `require.resolve`, we need to use `requireResolve: false` to preserve
				// the `resolve` method and `requireAsExpression: false` to preserve `require`.
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
