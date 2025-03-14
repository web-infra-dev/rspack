const mockFn = jest.fn();

let compilation1;
let compilation2;

class MyPlugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			mockFn();
			if (!compilation1) {
				compilation1 = compilation;
			} else {
				compilation2 = compilation;
			}
		});
	}
}

/** @type {import('../..').TCompilerCaseConfig} */
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

		expect(typeof compilation1).toBe("object");
		expect(typeof compilation2).toBe("object");
		expect(compilation1 !== compilation2).toBe(true);
		expect(compilation1.hash !== compilation2.hash).toBe(true);
	}
};
