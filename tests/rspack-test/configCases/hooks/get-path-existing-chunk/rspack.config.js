const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = false;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			compilation.hooks.processAssets.tap(pluginName, () => {
				called = true;
				const mainChunk = Array.from(compilation.chunks).find(
					chunk => chunk.name === "main"
				);
				expect(
					compilation.getPath("[id]-[name]-[chunkhash]-[contenthash]", {
						chunk: mainChunk,
						contentHashType: "javascript"
					})
				).toBe(
					`${mainChunk.id}-${mainChunk.name}-${mainChunk.renderedHash}-${mainChunk.contentHash["javascript"].slice(0, 20)}`
				);
			});
		});
		compiler.hooks.done.tap(pluginName, stats => {
			let json = stats.toJson();
			expect(json.errors.length === 0);
			expect(called).toBe(true);
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
