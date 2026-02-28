/** @type {import("@rspack/core").Configuration} */
module.exports = {
	output: {
		bundlerInfo: {
			force: true
		}
	},
	plugins: [
		compiler => {
			compiler.hooks.compilation.tap("test", compilation => {
				compilation.hooks.additionalTreeRuntimeRequirements.tap(
					"test",
					(_, set) => { }
				);
			});
		}
	],
};
