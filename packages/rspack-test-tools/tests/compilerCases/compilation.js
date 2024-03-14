const { ECompilerType } = require("../..");
const mockFn = jest.fn();

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			mockFn();
		});
	}
}

module.exports = {
	description: "should be called every compilation",
	name: __filename,
	compilerType: ECompilerType.Rspack,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.build(() => {
				compiler.build(() => {
					resolve();
				});
			});
		});
	},
	async check() {
		expect(mockFn).toBeCalledTimes(2);
	}
};
