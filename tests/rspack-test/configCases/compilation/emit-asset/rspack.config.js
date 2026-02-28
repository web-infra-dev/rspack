const { rspack } = require("@rspack/core");

const PLUGIN_NAME = "plugin";

const SYMBOL = Symbol("mark");

class Plugin {
	/**
	 * @param {import("@rspack/core").Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.compilation.tap(PLUGIN_NAME, (compilation) => {
			compilation.hooks.processAssets.tap(
				{
					name: PLUGIN_NAME,
					stage: rspack.Compilation.PROCESS_ASSETS_STAGE_ADDITIONS,
				},
				() => {
					compilation.emitAsset("/foo.txt", new compiler.webpack.sources.RawSource("foo"), {
						bool: true,
						number: 1,
						string: "foo",
						array: ["foo", "bar"],
						object: {
							bool: true,
							number: 1,
							string: "foo",
							array: ["foo", "bar"],
						},
						[SYMBOL]: "foo",
					});
				}
			);

			compilation.hooks.afterProcessAssets.tap(PLUGIN_NAME, () => {
				const { info } = compilation.getAsset("/foo.txt");

				expect(info.bool).toBe(true);
				expect(info.number).toBe(1);
				expect(info.string).toBe("foo");
				expect(info.array).toEqual(["foo", "bar"]);
				expect(info.object).toEqual({
					bool: true,
					number: 1,
					string: "foo",
					array: ["foo", "bar"],
				});
				expect(info[SYMBOL]).toBe("foo");
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	entry: "./index.js",
	plugins: [new Plugin()]
};
