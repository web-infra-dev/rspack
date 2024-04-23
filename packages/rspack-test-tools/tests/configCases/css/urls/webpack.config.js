module.exports = {
	output: {
		cssChunkFilename: "bundle.css"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	}
};
