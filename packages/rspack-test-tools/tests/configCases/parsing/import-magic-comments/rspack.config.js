/** @type {import("../../../../types").Configuration} */
module.exports = {
	entry: {
		index: "./index.js",
		test: "./test.js"
	},
	output: {
		filename: "[name].js"
	},
	module: {
		parser: {
			javascript: {
				importMagicComments: false
			}
		}
	}
};
