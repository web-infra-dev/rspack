/** @type {import("../../../../").Configuration} */
module.exports = {
	target: "node",
	output: {
		filename: "[name].js"
	},
	entry: "./index.js",
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			cacheGroups: {
				splittedFoo: {
					name: "splittedFoo",
					test: /(foo|foo-2)\.js/,
					priority: 0,
					reuseExistingChunk: false
				}
			}
		}
	}
};
