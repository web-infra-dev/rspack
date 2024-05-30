/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		concatenateModules: true,
		sideEffects: true,
		providedExports: true,
		usedExports: "global"
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
