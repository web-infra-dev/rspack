let step = 0;
let factorizeRequests = [];

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		function (compiler) {
			const PLUGIN_NAME = "TEST_PLUGIN";
			const { EntryPlugin } = compiler.webpack;
			compiler.hooks.finishMake.tapPromise(PLUGIN_NAME, compilation => {
				return new Promise((resolve, reject) => {
					compilation.addInclude(
						compiler.context,
						EntryPlugin.createDependency("./foo.js"),
						{},
						err => {
							if (err) return reject(err);
							return resolve();
						}
					);
				});
			});

			compiler.hooks.compilation.tap(
				PLUGIN_NAME,
				(compilation, { normalModuleFactory }) => {
					normalModuleFactory.hooks.factorize.tap(PLUGIN_NAME, data => {
						factorizeRequests.push(data.request);
					});
				}
			);
			compiler.hooks.done.tap(PLUGIN_NAME, () => {
				if (step === 0) {
					expect(factorizeRequests.length).toBe(2);
					expect(factorizeRequests.includes("./index.js")).toBe(true);
					expect(factorizeRequests.includes("./foo.js")).toBe(true);
				} else if (step === 1) {
					expect(factorizeRequests.length).toBe(1);
					expect(factorizeRequests.includes("./index.js")).toBe(true);
				} else {
					throw new Error("Unexpected step");
				}
				step += 1;
				factorizeRequests = [];
			});
		}
	]
};
