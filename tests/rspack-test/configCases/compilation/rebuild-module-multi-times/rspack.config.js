const pluginName = "plugin";

class Plugin {
	apply(compiler) {
		let initial = true;
		compiler.hooks.compilation.tap(pluginName, compilation => {
			compilation.hooks.finishModules.tapPromise(pluginName, async modules => {
				modules = [...modules];
				if (initial) {
					initial = false;

					const results = await Promise.all(
						modules.map(m => {
							return new Promise((resolve, reject) => {
								compilation.rebuildModule(m, (err, module) => {
									if (err) {
										reject(err);
									} else {
										resolve(module);
									}
								});
							});
						})
					);

					// should compile success
					expect(results.length).toBe(4);
				}
			});
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	module: {
		rules: [
			{
				test: /a\.js$/,
				use: [
					{
						loader: "./loader"
					}
				]
			}
		]
	},
	plugins: [new Plugin()]
};
