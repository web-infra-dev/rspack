/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization: {
		sideEffects: true
	},
	builtins: {
		define: {
			"process.env.NODE_ENV": "'production'"
		}
	}
};
