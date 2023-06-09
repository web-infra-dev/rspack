const assert = require("assert").strict;
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let called = 0;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			compilation.hooks.assetPath.tap(pluginName, (path, data, assetInfo) => {
				called++;
				if (path.includes("[name].js") && data.runtime === "other") {
					path = path.replace(".js", ".modified.js");
				}
				return path;
			});
		});
		compiler.hooks.done.tap(pluginName, stats => {
			let json = stats.toJson();
			assert(json.errors.length === 0, `${json.errors}`);
			assert(json.assets.length === 2, "Unexpected amount of files.");
			assert(
				json.assets.some(a => a.name === "other.modified.js"),
				"'other.modified.js' was not found"
			);
			assert(
				json.assets.some(a => a.name === "main.js"),
				"'main.js' was not found"
			);
		});
	}
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js",
		other: "./b.js"
	},
	module: {
		rules: []
	},
	plugins: [new Plugin()]
};
