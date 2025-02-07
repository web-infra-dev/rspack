/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.afterEnvironment.tap('getResolver', () => {
					expect(compiler.resolverFactory).toBeTruthy();
				})
			}
		}
	]
}
