/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	target: "web",
	node: false,
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			}
		}
	},
	experiments: { css: true },
	output: {
		filename: "bundle0.js?hash=[contenthash]",
		cssFilename: "bundle0.css?hash=[contenthash]"
	},
	optimization: {
		minimize: true
	}
};
