const { ECompilerType } = require("../..");

class MyPlugin {
	apply(compiler) {
		compiler.hooks.compilation.tap("Plugin", compilation => {
			compilation.hooks.processAssets.tap("Plugin", () => {
				const oldSource = compilation.assets["main.js"];
				expect(oldSource).toBeTruthy();
				expect(oldSource.source().includes("This is d")).toBeTruthy();
				const { RawSource } = require("webpack-sources");
				const updatedSource = new RawSource(
					`module.exports = "This is the updated d"`
				);
				compilation.updateAsset(
					"main.js",
					source => {
						expect(source.buffer()).toEqual(oldSource.buffer());
						return updatedSource;
					},
					_ => _
				);

				const newSource = compilation.assets["main.js"];
				expect(newSource).toBeTruthy();
				expect(newSource.buffer()).toStrictEqual(updatedSource.buffer());
			});
		});
	}
}

module.exports = {
	description: "should update assets",
	name: __filename,
	compilerType: ECompilerType.Rspack,
	options(context) {
		return {
			context: context.getSource(),
			entry: "./d",
			plugins: [new MyPlugin()]
		};
	}
};
