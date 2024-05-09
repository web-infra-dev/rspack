const { WEBPACK_SERVE } = process.env;
module.exports = /** @type {import('@rspack/cli').Configuration} */ {
	mode: "production",
	entry: "./entry.js",
	output: { clean: true },
	plugins: [
		{
			apply(compiler) {
				new compiler.webpack.DefinePlugin({
					DEFINE_ME: JSON.stringify(
						`WEBPACK_SERVE=${WEBPACK_SERVE ?? "<EMPTY>"}`
					)
				}).apply(compiler);
			}
		}
	],
	devServer: {
		devMiddleware: {
			writeToDisk: true
		}
	}
};
