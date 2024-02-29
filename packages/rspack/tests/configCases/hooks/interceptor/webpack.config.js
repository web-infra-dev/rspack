const { deepEqual } = require("assert");

class InterceptPlugin {
	apply(compiler) {
		const content = [];
		compiler.hooks.compilation.intercept({
			call() {
				content.push("compiler.hooks.compilation.intercept.call")
			}
		})
		compiler.hooks.done.tap(InterceptPlugin.name, () => {
			deepEqual(content, [
				"compiler.hooks.compilation.intercept.call"
			]);
		});
	}
}

/** @type {import('@rspack/cli').Configuration} */
const config = {
	plugins: [new InterceptPlugin()]
};
module.exports = config;
