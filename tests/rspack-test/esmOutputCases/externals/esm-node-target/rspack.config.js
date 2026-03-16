/** @type {import("@rspack/core").Configuration} */
module.exports = {
	externals: {},
	plugins: [
		{
			apply(compiler) {
				compiler.__internal__registerBuiltinPlugin({
					name: "EsmNodeTargetPlugin",
					options: false
				});
			}
		}
	]
};
