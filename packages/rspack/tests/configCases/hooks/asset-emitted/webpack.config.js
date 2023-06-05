const path = require("path");
const assert = require("assert").strict;
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let hasMainJs = false;
		compiler.hooks.assetEmitted.tap(pluginName, (filename, info) => {
			if (filename === "main.js") {
				assert(info.outputPath === path.resolve(__dirname, "dist"));
				assert(info.targetPath === path.resolve(__dirname, "dist/main.js"));
				assert(info.content.toString().includes("expect(3).toBe(3)"));
				hasMainJs = true;
			}
		});
		compiler.hooks.done.tap(pluginName, () => {
			assert(hasMainJs);
		});
	}
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
