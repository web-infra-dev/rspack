const { TreeShakingSharedPlugin } = require("@rspack/core").sharing;
const path = require("path");

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	// entry:'./index.js',
	optimization: {
		minimize: true,
		chunkIds: "named",
		moduleIds: "named"
	},
	output: {
		chunkFilename: "[id].js"
	},
	plugins: [
		new TreeShakingSharedPlugin({
			reShake: true,
			mfConfig: {
				name: 'reshake_share',
				library: {
					type: 'commonjs2',
				},
				shared: {
					'ui-lib': {
            version:'1.0.0',
						treeShaking: { 
              mode:'runtime-infer',
              usedExports:['Badge','MessagePro']
            },
						requiredVersion: '^1.0.0',
					},
					'ui-lib-dep': {
            version:'1.0.0',
						treeShaking: { 
              mode:'runtime-infer',
              usedExports:['Message']
            },
						requiredVersion: '^1.0.0',
					}
				},
        treeShakingSharedPlugins:[path.resolve(__dirname, './CustomPlugin.js')]
			}
		})
	]
};
