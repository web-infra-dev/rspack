const assert = require("assert").strict;
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = 0;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			compilation.hooks.chunkAsset.tap(pluginName, (chunk, name) => {
				assert(chunk.files.includes("bundle0.js"));
				called++;
			});
		});
		compiler.hooks.done.tap(pluginName, stats => {
			let json = stats.toJson();
			assert(json.errors.length === 0, `${json.errors}`);
			assert(called === 1);
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: []
	},
	plugins: [new Plugin()]
};
