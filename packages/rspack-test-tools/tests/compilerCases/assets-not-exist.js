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
					expect(err).toMatchInlineSnapshot(`
				Object {
				  "code": "GenericFailure",
				  "message": "Called Compilation.updateAsset for not existing filename something-else.js",
				  "stack": "Error: Called Compilation.updateAsset for not existing filename something-else.js/n    at _Compilation.updateAsset (<WORKSPACE>/rspack/dist/index.js<LINE_COL>)/n    at Object.fn (<ROOT>/tests/compilerCases/assets-not-exist.js<LINE_COL>)/n    at next (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at AsyncSeriesHook.callAsyncStageRange (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>/n    at new Promise (<anonymous>)/n    at AsyncSeriesHook.promiseStageRange (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at QueriedHook.promise (<HOME>/rspack-dev/rspack/node_modules/<PNPM_INNER>/@rspack/lite-tapable/dist/index.js<LINE_COL>)/n    at <WORKSPACE>/rspack/dist/index.js<LINE_COL>/n    at last.function (<WORKSPACE>/rspack/dist/index.js<LINE_COL>)",
				}
			`);
				}
			});
		});
	}
}

/** @type {import('../..').TCompilerCaseConfig} */
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
