const { createFsFromVolume, Volume } = require("memfs");
let statsJson;

class TestPlugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap(TestPlugin.name, compilation => {
			compilation.hooks.processAssets.tapAsync(
				TestPlugin.name,
				async (assets, callback) => {
					const child = compiler.createChildCompiler(
						compilation,
						"TestChild",
						1,
						compilation.outputOptions,
						[
							new compiler.webpack.EntryPlugin(
								compiler.context,
								"./fixtures/abc",
								{ name: "TestChild" }
							)
						]
					);
					child.runAsChild(err => callback(err));
				}
			);
		});
		compiler.hooks.done.tap("test plugin", stats => {
			statsJson = stats.toJson({
				all: false,
				children: true,
				assets: true
			});
		});
	}
}

/** @type {import('../..').TStatsAPICaseConfig} */
module.exports = {
	description: "should have children when using childCompiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./fixtures/a",
			plugins: [new TestPlugin()]
		};
	},
	async check(stats) {
		expect(statsJson).toMatchInlineSnapshot(`undefined`);
	}
};
