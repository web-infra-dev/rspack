class MyStillValidModulePlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyStillValidModulePlugin", compilation => {
			compilation.hooks.stillValidModule.tap(
				"MyStillValidModulePlugin",
				module => {
					console.log("this module is valid and not need rebuild");
					console.log(module);
				}
			);
		});
	}
}

module.exports = MyStillValidModulePlugin;
