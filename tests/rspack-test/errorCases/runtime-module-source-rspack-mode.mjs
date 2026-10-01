import path from "node:path";

/** @type {import('@rspack/test-tools').TErrorCaseConfig} */
export default {
	description:
		"should reject runtime module source changes from JS hooks in rspack mode",
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
			context: path.resolve(import.meta.dirname, "../configCases/hooks/runtime-module"),
			experiments: {
				runtimeMode: "rspack"
			},
			plugins: [new Plugin()]
		};
	},
	async check(diagnostics) {
		expect(diagnostics.errors).toHaveLength(1);
		expect(diagnostics.errors[0].message).toContain(
			'Compilation.hooks.runtimeModule source modifications are not supported when experiments.runtimeMode is "rspack"'
		);
		expect(diagnostics.warnings).toHaveLength(0);
	}
};
