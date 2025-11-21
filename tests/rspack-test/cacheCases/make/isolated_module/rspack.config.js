const rspack = require("@rspack/core");

let index = 0;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: () => `./entry${index}.js`,
	optimization: {
		minimize: false
	},
	experiments: {
		cache: {
			type: "persistent"
		}
	},
	plugins: [
		{
			apply(compiler) {
				index++;
				compiler.hooks.done.tapPromise("PLUGIN", async stats => {
					const { modules } = stats.toJson({ modules: true });
					const entry = modules.filter(item =>
						/entry[0-9]\.js$/.test(item.identifier)
					);
					expect(entry.length).toBe(1);
					expect(entry[0].identifier.endsWith(`entry${index}.js`)).toBe(true);
				});
			}
		}
	]
};
