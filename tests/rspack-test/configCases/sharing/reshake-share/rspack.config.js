const { TreeShakeSharedPlugin } = require("@rspack/core").sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	// entry:'./index.js',
	optimization: {
		// minimize:false,
		chunkIds: "named",
		moduleIds: "named"
	},
	output: {
		chunkFilename: "[id].js"
	},
	plugins: [
		new TreeShakeSharedPlugin({
			reshake: true,
					plugins: [
					{
						apply(compiler) {
							compiler.hooks.thisCompilation.tap('applyPlugins', (compilation) => {
								compilation.hooks.processAssets.tapPromise(
									{
										name: 'applyPlugins',
									},
									async () => {
										compilation.emitAsset('apply-plugin.json', new compilation.compiler.rspack.sources.RawSource(JSON.stringify({
											reshake: true
										})))
									})
							})
						}
					}
				],
			mfConfig: {
				name: 'reshake_share',
				library: {
					type: 'commonjs2',
				},
				shared: {
					'ui-lib': {
						treeshake: true,
						requiredVersion: '*',
						usedExports:['Badge','MessagePro']
					},
					'ui-lib-dep': {
						treeshake: true,
						requiredVersion: '*',
						usedExports:['Message']
					}
				},

			}
		})
	]
};
