/** @type {import("@rspack/core").Configuration} */
module.exports = {
    plugins: [
        compiler => {
            compiler.hooks.compilation.tap("PLUGIN", compilation => {
                compiler.hooks.afterCompile.tap("PLUGIN", () => {
                    const stats = compilation.getStats();
                    const { chunks } = stats.toJson({
                        all: false,
                        chunks: true
                    });
                    // Ensure that HotUpdateChunk is not added to chunks
                    expect(chunks.length).toBe(1);
                    expect(chunks[0].runtime[0]).toBe('main');
                    expect(chunks[0].files[0]).toBe('bundle.js');
                })
            })
        }
    ]
};
