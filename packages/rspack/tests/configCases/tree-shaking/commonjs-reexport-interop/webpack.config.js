/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		moduleIds: "named",
		minimize: false
	}
};
