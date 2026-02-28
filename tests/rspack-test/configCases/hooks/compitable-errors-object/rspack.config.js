const assert = require("assert");
class ErrorPlugin {
	apply(compiler) {
		compiler.hooks.thisCompilation.tap("DummyPlugin", compilation => {
			let error = new Error("error test");
			compilation.errors.push(error);
			let tempError = [...compilation.errors];
			assert(tempError.length === 1);
		});
	}
}
/**
 * @type {import("@rspack/core").Configuration}
 */
module.exports = {
	stats: "errors-warnings",
	plugins: [new ErrorPlugin()]
};
