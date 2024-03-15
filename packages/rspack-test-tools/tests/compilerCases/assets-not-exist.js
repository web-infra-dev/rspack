const mockFn = jest.fn();

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			compilation.hooks.processAssets.tap("Plugin", () => {
				const { RawSource } = require("webpack-sources");
				try {
					compilation.updateAsset(
						"something-else.js",
						new RawSource(`module.exports = "something-else"`),
						{
							minimized: true,
							development: true,
							related: {},
							hotModuleReplacement: false
						}
					);
				} catch (err) {
					mockFn();
					expect(err).toMatchInlineSnapshot(
						`[Error: Called Compilation.updateAsset for not existing filename something-else.js]`
					);
				}
			});
		});
	}
}

module.exports = {
	description: "should throw if the asset to be updated is not exist",
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	},
	async check() {
		expect(mockFn).toHaveBeenCalled();
	}
};
