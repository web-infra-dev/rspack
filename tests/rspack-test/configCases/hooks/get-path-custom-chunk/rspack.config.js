const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = false;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			called = true;
			expect(
				compilation.getPath("[id]-[name]-[chunkhash]-[contenthash]", {
					chunk: {
						name: "chunkname",
						id: "chunkid",
						hash: "chunkhash",
						contentHash: {
							javascript: "contenthash"
						}
					},
					contentHashType: "javascript"
				})
			).toBe("chunkid-chunkname-chunkhash-contenthash");

			expect(
				compilation.getPath("[name]-[chunkhash]-[contenthash]", {
					chunk: {
						id: "chunkid",
						hash: "chunkhash",
						contentHash: {
							javascript: "contenthash"
						}
					},
					contentHashType: "javascript"
				})
			).toBe("chunkid-chunkhash-contenthash");
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
