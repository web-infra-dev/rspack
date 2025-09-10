/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.afterEnvironment.tap("getResolver", () => {
					expect(compiler.resolverFactory).toBeTruthy();
				});
			}
		}
	]
};
