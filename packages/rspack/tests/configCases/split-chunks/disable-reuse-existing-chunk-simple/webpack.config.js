/** @type {import("../../../../dist").Configuration} */
module.exports = {
	target: "node",
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	experiments: {
		newSplitChunks: true
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			cacheGroups: {
				splittedFoo: {
					test: /(foo|foo-2)\.js/,
					priority: 0,
					reuseExistingChunk: false
				}
			}
		}
	}
};
