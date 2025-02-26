const { strict } = require("assert");
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let names = [];
		compiler.hooks.finishMake.tap(pluginName, compilation => {
			compilation.hooks.processAssets.tap(pluginName, () => {
				for (const m of compilation.modules) {
					names.push(m._constructorName);
				}
			});
		});
		compiler.hooks.done.tap(pluginName, () => {
			strict(names.filter(n => n === "NormalModule").length === 3);
			strict(names.filter(n => n === "ConcatenatedModule").length === 1);
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	plugins: [new Plugin()],
	optimization: {
		concatenateModules: true
	}
};
