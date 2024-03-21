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
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async build(_, compiler) {
		await new Promise(resolve => {
			compiler.run(() => {
				compiler.run(() => {
					resolve();
				});
			});
		});
	},
	async check() {
		expect(mockFn).toBeCalledTimes(2);
	}
};
