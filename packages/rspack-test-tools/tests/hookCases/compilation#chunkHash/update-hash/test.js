let updatedChunkHash = false;

/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: "should work with compilation#chunkHash",
	options(context) {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.compilation.tap("plugin", (compilation) => {
							compilation.hooks.chunkHash.tap("plugin", context.snapped((chunk, hash) => {
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
