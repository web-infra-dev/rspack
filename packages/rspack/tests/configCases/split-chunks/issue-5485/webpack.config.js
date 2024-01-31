/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	entry: {
		main: "./src/index.js"
	},
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			chunks: "all",
			minSize: 0,
			minChunks: 1,
			cacheGroups: {
				lib_1: {
					test: /lib\/a.js/,
					name: "lib1"
				},
				lib_2: {
					test: /lib\/a.js/,
					name: "lib2"
				}
			}
		}
	}
};
