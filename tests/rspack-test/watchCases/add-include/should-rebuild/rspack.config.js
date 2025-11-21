const rspack = require("@rspack/core");
const path = require("path");

/**@type {import('@rspack/core').Configuration} */
const config = {
	entry: {
		main: "./index.js"
	},
	mode: "development",
	plugins: [
		{
			apply(
				/**@type {import('@rspack/core').Compiler} */
				compiler
			) {
				let initial = true;

				compiler.hooks.finishMake.tapPromise("test", async compilation => {
					if (initial) {
						initial = false;
						return Promise.resolve();
					}

					return new Promise((resolve, reject) => {
						const dependency = rspack.EntryPlugin.createDependency(
							path.resolve(__dirname, "./plugin-included.js")
						);

						compilation.addInclude(
							compiler.context,
							dependency,
							{ name: "main" },
							err => {
								if (err) {
									reject(new Error(`Error adding entry: ${err}`));
								} else {
									resolve();
								}
							}
						);
					});
				});
			}
		}
	],
	experiments: {
		incremental: true
	}
};

module.exports = config;
