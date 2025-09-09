/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	target: "node",
	entry: {
		main: "./src/index.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minChunks: 1,
			minSize: 0,
			cacheGroups: {
				lib_1: {
					test: /lib[\/\\]a.js/,
					name: "lib1"
				},
				lib_2: {
					test: /lib[\/\\]a.js/,
					name: "lib2"
				}
			}
		}
	}
};
