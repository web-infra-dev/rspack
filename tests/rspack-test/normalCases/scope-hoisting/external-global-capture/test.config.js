/** @type {import("@rspack/core").Configuration} */
module.exports = {
	moduleScope(scope) {
		scope.React = { version: "global" };
	},
	externals: {
		react: "var React"
	},
	optimization: {
		concatenateModules: true
	}
};
