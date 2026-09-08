const compilers = [];

module.exports = {
	moduleScope(scope) {
		scope.recordCompiler = index => compilers.push(index);
		return scope;
	},
	afterExecute() {
		expect(compilers).toEqual([0, 1]);
	}
};
