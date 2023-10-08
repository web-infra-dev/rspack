/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: "node",
	entry: "./index.js",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			chunks: "all",
			name: "overall-foo",
			cacheGroups: {
				foo: {
					test: /foo\.js/,
					priority: 0
				},
				foo2: {
					test: /foo-2\.js/,
					priority: 0
				}
			}
		}
	}
};
