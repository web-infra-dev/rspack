// TODO: move to webpack-test after merged into webpack

/** @type {import("../../../../types").Configuration} */
module.exports = {
	entry: {
		bundle0: "./index.js",
		test: "./test.js"
	},
	module: {
		parser: {
			javascript: {
				requireDynamic: false
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
