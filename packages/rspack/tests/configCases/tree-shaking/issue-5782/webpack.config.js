/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	optimization: {
		innerGraph: true,
		sideEffects: true,
		usedExports: true,
		providedExports: true
	}
};
