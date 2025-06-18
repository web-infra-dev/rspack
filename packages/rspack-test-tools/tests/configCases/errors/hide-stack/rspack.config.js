/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: "./index",
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.done.tap("TestPlugin", stats => {
					const errors = stats.toJson({ errors: true }).errors;
					for (let error of errors) {
						if (error.message.includes("hide")) {
							expect(typeof error.details).toBe("string");
							expect(error.message.includes("stack")).toBeFalsy;
						} else {
							expect(typeof error.details).toBe("undefined");
						}
					}
				});
			}
		}
	]
};
