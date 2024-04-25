const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = false;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			const p = compilation.getPath("[contenthash]", {
				contentHash: "xxx1",
				chunk: {
					contentHash: "xxx2"
				}
			});
			called = true;
			expect(p).toBe("xxx1")
		});
		compiler.hooks.done.tap(pluginName, stats => {
			let json = stats.toJson();
			expect(json.errors.length === 0);
			expect(called).toBe(true);
		});
	}
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
