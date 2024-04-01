/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: {
			import: ["./index"]
		}
	},
	target: "node",
	output: {
		filename: "[name].js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
			"css": {
				exportsOnly: false,
			},
			"css/module": {
				exportsOnly: false,
			}
		},
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
