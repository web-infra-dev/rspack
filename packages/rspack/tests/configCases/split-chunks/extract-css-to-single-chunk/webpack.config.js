/** @type {import("../../../../").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./shims.js", "./index"]
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
