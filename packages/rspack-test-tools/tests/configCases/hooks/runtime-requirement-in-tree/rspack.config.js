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

			compilation.hooks.runtimeRequirementInTree
				.for(RuntimeGlobals.chunkName)
				.tap("TestFakePlugin", (chunk, set) => {
					expect(chunk.name).toBe("main");
					expect(set.has(RuntimeGlobals.chunkName)).toBeTruthy();
					expect(set.has(RuntimeGlobals.ensureChunk)).toBeTruthy();
					expect(set.has(RuntimeGlobals.ensureChunkHandlers)).toBeTruthy();
				});

			compilation.hooks.runtimeRequirementInTree
				.for(RuntimeGlobals.hasOwnProperty)
				.tap("TestFakePlugin", (chunk, set) => {
					expect(chunk.name).toBe("main");
					expect(set.has(RuntimeGlobals.hasOwnProperty)).toBeTruthy();
				});
		});
	}
}
/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
