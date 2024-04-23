module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false
			},
		},
	},
	devtool: "source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
