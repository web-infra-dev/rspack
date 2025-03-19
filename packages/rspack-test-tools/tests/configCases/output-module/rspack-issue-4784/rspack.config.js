const { RawSource } = require("webpack-sources");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	// USE `development` as `production` will be failed.
	// See: https://github.com/web-infra-dev/rspack/issues/5738
	mode: "development",
	entry: {
		main: "./index.js",
		m: "./m.js"
	},
	output: {
		filename: "[name].mjs",
		chunkFormat: "module",
		chunkLoading: "import",
		library: {
			type: "module"
		}
	},
	experiments: {
		outputModule: true
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.thisCompilation.tap("test", compilation => {
					compilation.hooks.processAssets.tap("test", assets => {
						compilation.updateAsset(
							"m.mjs",
							new RawSource(`import { a, b } from './main.mjs';
it('should get correctly exports', () => {
	expect(a).toBe('a')
	expect(b).toBe('b')
})`)
						);
					});
				});
			}
		}
	]
};
