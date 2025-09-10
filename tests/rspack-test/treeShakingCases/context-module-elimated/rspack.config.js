/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true
	},
	builtins: {
		treeShaking: "module",
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
