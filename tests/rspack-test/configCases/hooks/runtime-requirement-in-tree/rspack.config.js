const { RuntimeGlobals } = require("@rspack/core");

class Plugin {
	/**
	 * @param {import('@rspack/core').Compiler} compiler
	 */
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("TestFakePlugin", compilation => {
			compilation.hooks.additionalTreeRuntimeRequirements.tap(
				"TestFakePlugin",
				(_, set) => {
					set.add(RuntimeGlobals.chunkName);
					set.add(RuntimeGlobals.ensureChunk);
					set.add(RuntimeGlobals.ensureChunkHandlers);
				}
			);

			const once = new Set();
			const customRuntimeGlobal = "__webpack_require__.custom";
			const { RuntimeModule } = compiler.webpack;

			class CustomRuntimeModule extends RuntimeModule {
				constructor() {
					super("CustomRuntimeModule", RuntimeModule.STAGE_NORMAL);
				}
				generate() {
					return `__webpack_require__.custom = 42;`;
				}
			}

			compilation.hooks.runtimeRequirementInTree
				.for(RuntimeGlobals.chunkName)
				.tap("TestFakePlugin", (chunk, set) => {
					expect(once.has(RuntimeGlobals.chunkName)).toBe(false);
					once.add(RuntimeGlobals.chunkName);
					expect(chunk.name).toBe("main");
					expect(set.has(RuntimeGlobals.chunkName)).toBeTruthy();
					expect(set.has(RuntimeGlobals.ensureChunk)).toBeTruthy();
					expect(set.has(RuntimeGlobals.ensureChunkHandlers)).toBeTruthy();
				});

			compilation.hooks.runtimeRequirementInTree
				.for(RuntimeGlobals.hasOwnProperty)
				.tap("TestFakePlugin", (chunk, set) => {
					expect(once.has(RuntimeGlobals.hasOwnProperty)).toBe(false);
					once.add(RuntimeGlobals.chunkName);
					expect(chunk.name).toBe("main");
					expect(set.has(RuntimeGlobals.hasOwnProperty)).toBeTruthy();
					set.add(customRuntimeGlobal);
				});

			compilation.hooks.runtimeRequirementInTree
				.for(customRuntimeGlobal)
				.tap("TestFakePlugin", (chunk, set) => {
					expect(once.has(customRuntimeGlobal)).toBe(false);
					once.add(customRuntimeGlobal);
					compilation.addRuntimeModule(chunk, new CustomRuntimeModule());
				});
		});
	}
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	plugins: [new Plugin()]
};
