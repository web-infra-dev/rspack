let firstRun = true;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.compilation.tap("test", compilation => {
					compilation.hooks.seal.tap("test", () => {
						const builtModules = Array.from(compilation.builtModules).map(
							m => m.rawRequest
						);
						builtModules.sort();
						if (firstRun) {
							expect(builtModules).toEqual(["./foo", "./index.js"]);
							firstRun = false;
						} else {
							expect(builtModules).toEqual(["./foo"]);
						}
					});
				});
			}
		}
	]
};
