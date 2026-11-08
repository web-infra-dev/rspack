export default {
	moduleScope(scope) {
		scope.define = (...args) => args[args.length - 1]();
	}
};
