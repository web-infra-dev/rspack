/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		other: "./src/index"
	},
	plugins: [
		{
			apply(compiler) {
				const { RuntimeGlobals } = compiler.webpack;
				compiler.hooks.thisCompilation.tap("testPlugin", compilation => {
					compilation.hooks.additionalTreeRuntimeRequirements.tap(
						"testPlugin",
						(chunk, set) => {
							set.add(RuntimeGlobals.createScriptUrl);
						}
					);
				});
			}
		}
	]
};
