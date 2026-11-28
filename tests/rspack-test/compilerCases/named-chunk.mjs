const mockFn = rstest.fn();

class MyPlugin {
	apply(compiler) {
		compiler.hooks.afterCompile.tap("Plugin", compilation => {
			let c = compilation.namedChunks.get("d");
			expect(c.name).toBe("d");
			expect(compilation.namedChunks.has("d")).toBe(true);
			expect(compilation.namedChunks.has("missing")).toBe(false);
			expect(compilation.namedChunks.size).toBe(1);
			mockFn();
		});
	}
}

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
export default {
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
