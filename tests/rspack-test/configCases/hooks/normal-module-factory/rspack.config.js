/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.normalModuleFactory.tap("getResolver", nmf => {
					const resolver = nmf.getResolver("normal");
					expect(resolver).toBeTruthy();
				});
			}
		}
	]
};
