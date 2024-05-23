/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true,
		usedExports: true,
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
