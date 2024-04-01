module.exports = {
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
	devtool: "source-map",
	externals: ["source-map"],
	externalsType: "commonjs"
};
