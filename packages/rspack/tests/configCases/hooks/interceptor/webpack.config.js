const { deepEqual } = require("assert");

class InterceptPlugin {
	apply(compiler) {
		const content = [];
		compiler.hooks.beforeCompile.intercept({
			call() {
				content.push("compiler.hooks.beforeCompile.intercept.call")
			}
		})
		compiler.hooks.compile.intercept({
			call() {
				content.push("compiler.hooks.compile.intercept.call")
			}
		})
		compiler.hooks.finishMake.intercept({
			call() {
				content.push("compiler.hooks.finishMake.intercept.call")
			}
		})
		compiler.hooks.afterCompile.intercept({
			call() {
				content.push("compiler.hooks.afterCompile.intercept.call")
			}
		})
		compiler.hooks.done.tap(InterceptPlugin.name, () => {
			deepEqual(content, [
				"compiler.hooks.beforeCompile.intercept.call",
				"compiler.hooks.compile.intercept.call",
				"compiler.hooks.finishMake.intercept.call",
				"compiler.hooks.afterCompile.intercept.call",
			]);
		});
	}
}

/** @type {import('@rspack/cli').Configuration} */
const config = {
	plugins: [new InterceptPlugin()]
};
module.exports = config;
