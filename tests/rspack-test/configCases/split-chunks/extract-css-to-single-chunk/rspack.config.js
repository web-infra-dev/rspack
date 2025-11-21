/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./index"]
		}
	},
	node: {
		__dirname: false,
		__filename: false
	},
	target: "web",
	output: {
		filename: "[name].js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
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
