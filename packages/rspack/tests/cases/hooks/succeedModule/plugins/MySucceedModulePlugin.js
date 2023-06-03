class MySucceedModulePlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MySucceedModulePlugin", compilation => {
			compilation.hooks.succeedModule.tap("MySucceedModulePlugin", module => {
				console.log("trigger succeedModule hook success");
				console.log(module);
			});
		});
	}
}

module.exports = MySucceedModulePlugin;
