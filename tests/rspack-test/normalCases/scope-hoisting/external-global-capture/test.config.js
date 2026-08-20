/** @type {import("@rspack/core").Configuration} */
module.exports = {
	moduleScope(scope) {
		scope.React = { version: "global" };
	},
	optimization: {
		concatenateModules: true
	}
};
