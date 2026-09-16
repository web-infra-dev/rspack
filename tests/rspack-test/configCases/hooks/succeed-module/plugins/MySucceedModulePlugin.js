const mockFn = rstest.fn();

class MySucceedModulePlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MySucceedModulePlugin", compilation => {
			compilation.hooks.succeedModule.tap("MySucceedModulePlugin", module => {
				if (module.resource) {
					const source = module.originalSource();
					expect(source).not.toBeNull();
				}
				mockFn();
			});
		});
		compiler.hooks.done.tap("MySucceedModulePlugin", () => {
			expect(mockFn).toBeCalledTimes(4);
		});
	}
}

module.exports = MySucceedModulePlugin;
