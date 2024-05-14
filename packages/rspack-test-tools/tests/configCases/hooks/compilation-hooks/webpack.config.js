const { deepEqual, strict } = require("assert");
const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let order = [];

		compiler.hooks.beforeCompile.tap(pluginName, () => {
			order.push("hooks.beforeCompile");
		});
		compiler.hooks.make.tap(pluginName, () => {
			order.push("hooks.make");
		});
		compiler.hooks.finishMake.tap(pluginName, () => {
			order.push("hooks.finishMake");
		});
		compiler.hooks.afterCompile.tap(pluginName, () => {
			order.push("hooks.afterCompile");
		});

		compiler.hooks.done.tap(pluginName, stats => {
			let json = stats.toJson();
			strict(json.errors.length === 0, `${json.errors}`);
			deepEqual(order, [
				"hooks.beforeCompile",
				"hooks.make",
				"hooks.finishMake",
				"hooks.afterCompile"
			]);
		});
	}
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	module: {
		rules: []
	},
	plugins: [new Plugin()]
};
