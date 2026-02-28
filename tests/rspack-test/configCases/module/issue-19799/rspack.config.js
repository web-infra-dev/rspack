"use strict";

const path = require("path");
const { rspack } = require("@rspack/core");

/** @type {(env: Env, options: TestOptions) => import("@rspack/core").Configuration[]} */
module.exports = (env, { testPath }) => [
	{
		output: {
			filename: "lib.mjs",
			module: true,
			library: {
				type: "module"
			}
		},
		plugins: [
			{
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", (compilation) => {
						const hooks =
							rspack.javascript.JavascriptModulesPlugin.getCompilationHooks(
								compilation
							);
						hooks.inlineInRuntimeBailout.tap("test", () => "test bailout");
					});
				}
			}
		]
	},
	{
		name: "test-output",
		entry: "./test.js",
		output: {
			filename: "test.mjs",
			module: true
		},
		experiments: { },
		externals: {
			lib: path.resolve(testPath, "./lib.mjs")
		},
		externalsType: "module-import"
	}
];
