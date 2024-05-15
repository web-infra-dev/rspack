/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		mangleExports: "deterministic",
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
