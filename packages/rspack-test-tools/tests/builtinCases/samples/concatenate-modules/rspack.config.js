/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true,
		sideEffects: true,
		providedExports: true,
		usedExports: "global"
	},
	experiments: {
		rspackFuture: {
			newTreeshaking: true
		}
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
