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
								"./abc",
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

defineStatsAPICase(Utils.basename(__filename), {
	description: "should have children when using childCompiler",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./a",
			plugins: [new TestPlugin()]
		};
	},
	async check(stats) {
		const assets = statsJson.assets.map(i => i.name);
		assets.sort();
		expect(assets).toEqual([
			"TestChild.js",
			"main.js"
		]);

		const children = statsJson.children;
		expect(children.length).toBe(1);
		expect(children[0].name).toBe("TestChild");
		expect(children[0].assets.length).toBe(1);
		expect(children[0].assets[0].name).toBe("TestChild.js");
	}
});
