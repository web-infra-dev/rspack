/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	optimization: {
		innerGraph: true,
		sideEffects: true,
		usedExports: true,
		providedExports: true
	}
};
