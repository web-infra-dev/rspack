module.exports = {
	moduleScope(scope) {
		scope.window.document.defaultView = scope.window;
	}
};
