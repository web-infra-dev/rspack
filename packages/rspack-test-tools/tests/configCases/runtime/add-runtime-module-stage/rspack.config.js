const { RuntimeModule, RuntimeGlobals } = require("@rspack/core");

class MockNormalRuntimeModule extends RuntimeModule {
	constructor(chunk) {
		super("mock-normal", RuntimeModule.STAGE_NORMAL);
	}

	generate() {
		return `__webpack_require__.mockNormal = "normal";`;
	}
}

class MockTriggerRuntimeModule extends RuntimeModule {
	constructor(chunk) {
		super("mock-trigger", RuntimeModule.STAGE_TRIGGER);
	}

	generate() {
		return `__webpack_require__.mockTrigger = "trigger";`;
	}
}

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index.js",
	mode: "development",
	devtool: false,
	optimization: {
		minimize: false,
		sideEffects: false,
		concatenateModules: false,
		usedExports: false,
		innerGraph: false,
		providedExports: false
	},
	plugins: [
		/**
		 * @param {import('@rspack/core').Compiler} compiler
		 */
		compiler => {
			compiler.hooks.thisCompilation.tap("MockRuntimePlugin", compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"MockRuntimePlugin",
					(chunk, set) => {
						set.add(RuntimeGlobals.publicPath);
						set.add(RuntimeGlobals.getChunkScriptFilename);
						compilation.addRuntimeModule(chunk, new MockTriggerRuntimeModule());
						compilation.addRuntimeModule(chunk, new MockNormalRuntimeModule());
					}
				);
			});
		}
	]
};
