const { ECompilerType } = require("../..");

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			let cache = compilation.getCache("MyPlugin");
			expect(cache).not.toBeNull();
		});
	}
}

module.exports = {
	description: "should call getCache function correctly",
	name: __filename,
	compilerType: ECompilerType.Rspack,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
};
