/** @type {import("@rspack/core").Configuration} */
module.exports = {
	devtool: "source-map",
	module: {
		rules: [
			{
				test: /\.(jsx?|tsx?)$/,
				use: [
					{
						loader: "builtin:swc-loader"
					}
				]
			}
		]
	},
	plugins: [
		{
			apply(compiler) {
				compiler.hooks.afterEmit.tap("PLUGIN", compilation => {
					const sourceMap = JSON.parse(
						compilation.assets["bundle0.js.map"].source()
					);
					expect(sourceMap.sources).toEqual(
						expect.arrayContaining([
							"webpack:///./node_modules/lib-with-source-map/main.js",
							"webpack:///./index.js"
						])
					);
				});
			}
		}
	]
};
