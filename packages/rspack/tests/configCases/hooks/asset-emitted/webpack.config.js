const path = require("path");
const assert = require("assert").strict;
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let hasMainJs = false;
		compiler.hooks.assetEmitted.tap(pluginName, (filename, info) => {
			if (filename === "bundle0.js") {
				assert(info.targetPath.includes("bundle0.js"));
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
