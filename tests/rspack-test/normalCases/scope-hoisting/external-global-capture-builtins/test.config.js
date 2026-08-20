/** @type {import("@rspack/core").Configuration} */
module.exports = {
	moduleScope(scope) {
		scope.JSON = { marker: "global-json" };
		scope.Promise = { marker: "global-promise" };
		scope.URL = { marker: "global-url" };
		scope.URLSearchParams = { marker: "global-url-search-params" };
		scope.Symbol = { marker: "global-symbol" };
		scope.Reflect = { marker: "global-reflect" };
		scope.marker = "global-global-this";
	},
	optimization: {
		concatenateModules: true
	}
};
