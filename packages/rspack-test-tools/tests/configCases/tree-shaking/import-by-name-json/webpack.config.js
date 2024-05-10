/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	module: {
		rules: []
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		moduleIds: "named",
		minimize: false
	},
	externalsPresets: {
		node: true
	}
};
