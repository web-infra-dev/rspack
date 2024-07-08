/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		mangleExports: "deterministic",
		providedExports: true,
		usedExports: "global"
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
