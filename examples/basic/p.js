const name = "P";
class P {
	apply(compiler) {
		compiler.hooks.compilation.tap(name, (compilation) => {
			compilation.hooks.succeedModule.tap(name, function (module) {
				// console.log(module)
			});
		});
	}
}

module.exports = { P };
