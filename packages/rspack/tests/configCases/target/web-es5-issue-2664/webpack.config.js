const { ConcatSource, RawSource } = require("webpack-sources");

module.exports = {
	target: ["web", "es5"],
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("MyPlugin", compilation => {
					compilation.hooks.processAssets.tap("MyPlugin", assets => {
						const main = assets["main.js"];
						assets["main.js"] = new ConcatSource(
							new RawSource(
								";var __rspack_symbol__ = globalThis.Symbol; delete globalThis.Symbol;"
							),
							main,
							new RawSource(";globalThis.Symbol = __rspack_symbol__;")
						);
					});
				});
			}
		}
	]
};
