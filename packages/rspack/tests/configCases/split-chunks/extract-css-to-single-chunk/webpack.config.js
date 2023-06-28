/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./index"]
		}
	},
	experiments: {
		newSplitChunks: true
	},
	target: "node",
	output: {
		filename: "[name].js"
	},
	optimization: {
		splitChunks: {
			minSize: 1,
			cacheGroups: {
				styles: {
					chunks: "all",
					name: "styles",
					test: /\.css$/,
					priority: 99
				}
			}
		}
	}
};
