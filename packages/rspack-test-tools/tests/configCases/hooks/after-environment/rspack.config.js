/** @type {import("../../../..").THookCaseConfig} */
module.exports = {
	description: 'should access resolveFactory in afterEnvironment hook',
	options(context) {
		return {
			plugins: [
				{
					apply(compiler) {
						compiler.hooks.afterEnvironment.tap('getResolver', () => {
							expect(compiler.resolverFactory).toBeTruthy
						})
					}
				}
			]
		}
	}
}
