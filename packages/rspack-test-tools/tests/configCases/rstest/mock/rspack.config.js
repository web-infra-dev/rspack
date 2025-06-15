const { RstestPlugin } = require("@rspack/core");

class RstestSimpleRuntimePlugin {
	constructor() {}

	apply(compiler) {
		const { RuntimeModule, RuntimeGlobals } = compiler.rspack;
		class RetestImportRuntimeModule extends RuntimeModule {
			constructor() {
				super("rstest runtime");
			}

			generate() {
				return `
__webpack_require__.set_mock = (id, modFactory) => {
	__webpack_module_cache__[id] = { exports: modFactory() }
};`;
			}
		}

		compiler.hooks.thisCompilation.tap(
			"RstestSimpleRuntimePlugin",
			compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"RstestSimpleRuntimePlugin",
					chunk => {
						compilation.addRuntimeModule(
							chunk,
							new RetestImportRuntimeModule()
						);
					}
				);
			}
		);
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = [
	{
		entry: "./index.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			mangleExports: false
		},
		plugins: [
			new RstestSimpleRuntimePlugin(),
			new RstestPlugin({
				injectModulePathName: true
			})
		]
	},
	{
		entry: "./test.js",
		target: "node",
		node: {
			__filename: false,
			__dirname: false
		},
		optimization: {
			mangleExports: false
		}
	}
];
