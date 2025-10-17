const mockFn = jest.fn();

class MyStillValidModulePlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyStillValidModulePlugin", compilation => {
			compilation.hooks.stillValidModule.tap(
				"MyStillValidModulePlugin",
				module => {
					mockFn();
				}
			);
		});
		compiler.hooks.done.tap("MyStillValidModulePlugin", () => {
			expect(mockFn).toBeCalledTimes(0);
		});
	}
}

module.exports = MyStillValidModulePlugin;
