const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	experiments: {
		cache: {
			type: "persistent"
		},
		incremental: true
	},
	plugins: [
		{
			apply(compiler) {
				let require_a_times = 0;
				compiler.hooks.compilation.tap(
					"PLUGIN",
					(_, { normalModuleFactory }) => {
						// reset when build/rebuild start
						require_a_times = 0;
						normalModuleFactory.hooks.resolve.tapPromise(
							"PLUGIN",
							async resolveData => {
								if (resolveData.request === "./a.js") {
									resolveData.fileDependencies.push(
										path.join(__dirname, "./file.js")
									);
									require_a_times++;
								}
							}
						);
					}
				);
				compiler.hooks.done.tap("PLUGIN", () => {
					expect(require_a_times).toBe(1);
				});
			}
		}
	]
};
