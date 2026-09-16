const compilers = new Set();

module.exports = {
	moduleScope(scope) {
		// Record execution from inside each bundle's test, after the harness
		// supplies COMPILER_INDEX to its module scope.
		scope.recordCompiler = index => compilers.add(index);
		return scope;
	},
	afterExecute() {
		expect([...compilers]).toEqual([0, 1]);
	}
};
