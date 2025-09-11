/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	plugins: [
		function testPlugin(compiler) {
			compiler.hooks.compilation.tap("testPlugin", (compilation, { normalModuleFactory }) => {
				normalModuleFactory.hooks.resolveForScheme.for("data").tap("testPlugin", (resourceData) => {
					expect(resourceData.resource).toBe("data:text/javascript;charset=utf-8,export const value = 42;");
					expect(resourceData.path).toBe("data:text/javascript;charset=utf-8,export const value = 42;");
				});
			});
		}
	]
};
