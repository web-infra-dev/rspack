const assert = require("assert");

class Plugin {
	apply(compiler) {
		let count = 0;
		compiler.hooks.shouldEmit.tap("should-emit-should-works", compilation => {
			assert(typeof compilation !== "undefined");
			assert(typeof compilation.hooks !== "undefined");
			count += 1;
		});

		compiler.hooks.done.tap("check", () => {
			assert(count === 1);
		});
	}
}

/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
