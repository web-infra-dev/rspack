const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: [
		({ context, request, getResolve }, callback) => {
			const resolveFunction = getResolve();
			resolveFunction(context, request, (err, resource) => {
				if (err) {
					return callback(err);
				}
				if (
					resource === path.resolve(__dirname, "node_modules/foo/index.mjs")
				) {
					callback(null, "global esm");
				} else if (
					resource === path.resolve(__dirname, "node_modules/foo/index.cjs")
				) {
					callback(null, "global cjs");
				} else {
					callback(null, false);
				}
			});
		}
	]
};
