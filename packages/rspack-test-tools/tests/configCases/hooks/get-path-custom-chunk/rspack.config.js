const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = false;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			called = true;
			expect(compilation.getPath("[id]-[name]-[chunkhash]", {
				chunk: {
					name: "chunkname",
					id: "chunkid",
					hash: "chunkhash",
				}
			})).toBe("chunkid-chunkname-chunkhash");
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
