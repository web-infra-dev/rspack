export default {
	findBundle: function (i, options) {
		return ["main.js"];
	},
	moduleScope(scope) {
		scope["MyWorker"] = (...args) => new scope.Worker(...args);
	}
};
