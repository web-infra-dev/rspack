/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		compiler => {
			compiler.hooks.afterCompile.tap("PLUGIN", compilation => {
				const stats = compilation.getStats();
				const { chunks } = stats.toJson({
					all: false,
					chunks: true
				});
				// Ensure that HotUpdateChunk is not added to chunks
				expect(chunks.length).toBe(1);
				expect(chunks[0].runtime[0]).toBe("main");
				expect(chunks[0].files[0]).toBe("bundle.js");
			});
		}
	]
};
