const { ConcatSource } = require("webpack-sources");

/**
 * @type {import('@rspack/core').RspackOptions}
 */
module.exports = {
	entry: {
		main: "./index.js"
	},
	plugins: [
		{
			name: "test",
			apply(compiler) {
				compiler.hooks.compilation.tap("compilation", compilation => {
					compilation.hooks.processAssets.tapPromise(
						"processAssets1",
						async assets => {
							const inspect = new ConcatSource();
							for (const [n, cg] of compilation.entrypoints) {
								inspect.add(`entry name: ${n}\n`);
								for (const file of cg.getFiles()) {
									inspect.add(`  file: ${file}\n`);
								}
							}
							compilation.emitAsset("inspect.txt", inspect);
						}
					);
				});
			}
		}
	]
};
