const compilers = new Set();

module.exports = {
	moduleScope(scope) {
		scope.recordCompiler = index => compilers.add(index);
		return scope;
	},
	afterExecute() {
		// An explicit NEXT_START must not gain another automatic restart.
		expect([...compilers]).toEqual([0, 1]);
	}
};
