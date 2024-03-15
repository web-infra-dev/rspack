const mockFn = jest.fn();

class MyPlugin {
	apply(compiler) {
		compiler.hooks.afterCompile.tap("Plugin", compilation => {
			let c = compilation.namedChunks.get("d");
			expect(c.name).toBe("d");
			mockFn();
		});
	}
}

module.exports = {
	description: "should work with `namedChunks`",
	options(context) {
		return {
			context: context.getSource(),
			entry: {
				d: "./d"
			},
			plugins: [new MyPlugin()]
		};
	},
	async check() {
		expect(mockFn).toBeCalled();
	}
};
