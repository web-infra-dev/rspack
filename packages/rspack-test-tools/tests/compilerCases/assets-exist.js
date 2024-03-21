class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			compilation.hooks.processAssets.tap("Plugin", () => {
				const { RawSource } = require("webpack-sources");
				compilation.emitAsset(
					"main.js",
					new RawSource(`module.exports = "I'm the right main.js"`)
				);
			});
		});
	}
}

module.exports = {
	description: "should have error if the asset to be emitted is exist",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async check(_, __, stats) {
		expect(stats.toJson().errors[0].message).toMatchInlineSnapshot(`
			"  × Conflict: Multiple assets emit different content to the same filename main.js
			"
		`);
	}
};
