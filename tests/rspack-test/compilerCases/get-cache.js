const { basename, defineCompileCase } = require("@rspack/test-tools");
class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("MyPlugin", compilation => {
			let cache = compilation.getCache("MyPlugin");
			expect(cache).not.toBeNull();
		});
	}
}


defineCompileCase(Utils.basename(__filename), {
	description: "should call getCache function correctly",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
});
