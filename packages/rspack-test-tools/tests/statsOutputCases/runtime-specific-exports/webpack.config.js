
/** @type {import("../../../dist").Configuration} */
module.exports = {
	entry: "./example.js",
	optimization: {
		usedExports: true,
		providedExports: true,
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	stats: {
		usedExports: true,
		providedExports: true,
	}
};

