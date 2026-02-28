/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		module: true,
		filename: "[name].mjs",
		library: {
			type: "module"
		}
	},
	target: ["web", "node"],
	optimization: {
		minimize: true,
		runtimeChunk: "single",
		splitChunks: {
			cacheGroups: {
				separate: {
					test: /separate/,
					chunks: "all",
					filename: "separate.mjs",
					enforce: true
				}
			}
		}
	},
	externals: {
		"external-self": "./main.mjs"
	}
};
