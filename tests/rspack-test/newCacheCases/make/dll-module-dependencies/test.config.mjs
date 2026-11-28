export default {
	moduleScope(scope) {
		// The DLL exports a require function, so register the restart test here.
		if (scope.module) {
			scope.it("should retain DLL dependencies after the entry changes", async () => {
				if (scope.COMPILER_INDEX === 1) await scope.NEXT_START();
			});
		}
		return scope;
	}
};
