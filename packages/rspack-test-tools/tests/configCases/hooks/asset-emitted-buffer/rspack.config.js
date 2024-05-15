const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let hasEmittedAsset = false;
		let contentLength = -1;
		compiler.hooks.thisCompilation.tap(pluginName, compilation => {
			const { RawSource } = compiler.webpack.sources;
			compilation.hooks.processAssets.tap(pluginName, () => {
				const buffer = Buffer.from("i am content of emit asset");
				contentLength = buffer.length;
				const source = new RawSource(buffer, false);
				compilation.emitAsset('emit-asset.js', source);
			});
		});


		compiler.hooks.assetEmitted.tap(pluginName, (filename, info) => {
			if (filename === "emit-asset.js") {
				expect(info.targetPath.includes("emit-asset.js")).toBeTruthy();
				expect(info.content.length).toBe(contentLength);
				expect(info.content.toString('utf-8').includes("i am content of emit asset")).toBeTruthy();
				hasEmittedAsset = true;
			}
		});
		compiler.hooks.done.tap(pluginName, () => {
			expect(hasEmittedAsset).toBeTruthy();
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
