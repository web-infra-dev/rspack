const assert = require("assert");

class Plugin {
	apply(compiler) {
		let count = 0;
		compiler.hooks.compilation.tap("test", compilation => {
			assert(typeof compilation.hooks.afterProcessAssets !== "undefined");
			compilation.hooks.afterProcessAssets.tap(
				"should-emit-should-works",
				assets => {
					assert(typeof assets !== "undefined");
					assert(typeof assets["bundle0.js"] !== "undefined");
					count += 1;
				}
			);
		});

		compiler.hooks.done.tap("check", () => {
			assert(count === 1);
		});
	}
}

/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	context: __dirname,
	plugins: [new Plugin()]
};
