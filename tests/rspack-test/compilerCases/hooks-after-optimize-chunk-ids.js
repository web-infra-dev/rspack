let chunkIdsWhenCalled = [];

/** @type {import('@rspack/test-tools').TCompilerCaseConfig} */
module.exports = {
	description: "should call afterOptimizeChunkIds hook after chunk ids are assigned",
	options(context) {
		chunkIdsWhenCalled = [];
		return {
			context: context.getSource(),
			entry: "./chunks",
			plugins: [{
				apply(compiler) {
					compiler.hooks.compilation.tap("MyPlugin", compilation => {
						compilation.hooks.afterOptimizeChunkIds.tap("MyPlugin", chunks => {
							chunkIdsWhenCalled.push([...chunks].map(chunk => chunk.id));
						});
					});
				}
			}]
		};
	},
	async check() {
		expect(chunkIdsWhenCalled).toHaveLength(1);
		const ids = chunkIdsWhenCalled[0];
		expect(ids).toHaveLength(2);
		for (const id of ids) {
			expect(typeof id === "string" || typeof id === "number").toBe(true);
		}
	}
};
