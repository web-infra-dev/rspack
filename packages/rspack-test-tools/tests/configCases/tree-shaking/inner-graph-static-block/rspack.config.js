/**@type {import("@rspack/core").Configuration}*/
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
