/** @type {import("@rspack/core").Configuration} */
module.exports = {
	plugins: [
		compiler => {
			compiler.hooks.compilation.tap("test", compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"test",
					(_, set) => {}
				);
			});
		}
	],
	experiments: {
		rspackFuture: {
			bundlerInfo: {
				force: true
			}
		}
	}
};
