/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index",
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("TestPlugin", stats => {
					const errors = stats.toJson({
						errors: true,
						ids: true,
						moduleTrace: true
					}).errors;
					expect(errors.length).toBe(1);
					const moduleTrace = errors[0].moduleTrace;
					expect(moduleTrace[0].moduleName).toBe("./c.js");
					expect(moduleTrace[0].originName).toBe("./b.js");
					expect(moduleTrace[1].moduleName).toBe("./b.js");
					expect(moduleTrace[1].originName).toBe("./a.js");
					expect(moduleTrace[2].moduleName).toBe("./a.js");
					expect(moduleTrace[2].originName).toBe("./index.js");
				});
			}
		}
	]
};
