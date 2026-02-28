module.exports = {
	findBundle(i, config, step) {
		return ["vendors.js", "a.js", "b.js"]
	},
	moduleScope(scope) {
		scope.window.document.defaultView = scope.window;
	}
};
