const mockFn = jest.fn();

class MySucceedModulePlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MySucceedModulePlugin", compilation => {
			compilation.hooks.succeedModule.tap("MySucceedModulePlugin", module => {
				mockFn();
			});
		});
		compiler.hooks.done.tap("MySucceedModulePlugin", () => {
			expect(mockFn).toBeCalledTimes(4);
		});
	}
}

module.exports = MySucceedModulePlugin;
