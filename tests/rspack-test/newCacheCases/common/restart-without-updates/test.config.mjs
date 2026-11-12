const compilers = [];

export default {
	moduleScope(scope) {
		scope.recordCompiler = index => compilers.push(index);
		return scope;
	},
	afterExecute() {
		expect(compilers).toEqual([0, 1]);
	}
};
