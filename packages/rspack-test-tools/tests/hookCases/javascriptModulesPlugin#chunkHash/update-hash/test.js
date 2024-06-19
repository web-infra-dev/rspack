let updatedChunkHash = false;

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with javascriptModulesPlugin#chunkHash",
	options(context) {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.compilation.tap("plugin", (compilation) => {
							const hooks = compiler.webpack.javascript.JavascriptModulesPlugin.getCompilationHooks(compilation);
							hooks.chunkHash.tap("plugin", context.snapped((chunk, hash) => {
								updatedChunkHash = true;
							}))
						});
					}
				}
			]
		};
	},
	async check() {
		expect(updatedChunkHash).toBeTruthy();
	}
};
