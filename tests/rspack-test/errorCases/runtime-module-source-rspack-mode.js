const path = require("path");

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
module.exports = {
	description:
		"should support runtime module source changes from JS hooks in rspack mode",
	options() {
		let modified = false;
		class Plugin {
			apply(compiler) {
				compiler.hooks.compilation.tap("TestFakePlugin", compilation => {
					compilation.hooks.runtimeModule.tap("TestFakePlugin", module => {
						if (!modified && module.source?.source) {
							modified = true;
							const originSource = module.source.source.toString("utf-8");
							module.source.source = Buffer.from(
								`${originSource}\n__webpack_require__.test = true;\n`,
								"utf-8"
							);
						}
					});
				});
			}
		}

		return {
			entry: {
				main: "./index.js",
				chunk: "./chunk.js"
			},
			context: path.resolve(__dirname, "../configCases/hooks/runtime-module"),
			target: "node",
			experiments: {
				runtimeMode: "rspack"
			},
			plugins: [new Plugin()]
		};
	},
	async check(diagnostics) {
		expect(diagnostics.errors).toHaveLength(0);
		expect(diagnostics.warnings).toHaveLength(0);
	}
};
