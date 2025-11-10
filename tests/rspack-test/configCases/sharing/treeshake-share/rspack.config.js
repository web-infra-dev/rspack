const path = require("path");

const { sharing } = require("@rspack/core");

const { ShareContainerPlugin, OptimizeDependencyReferencedExportsPlugin,SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	optimization:{
		minimize: true,
		chunkIds:'named',
		moduleIds: 'named'
	},
	output: {
		chunkFilename: "[id].js"
	},
	entry: {
		main: "./index.js"
	},
	plugins: [
		new SharePlugin({
			shared: {
				'ui-lib': {
					requiredVersion:'*'
				}
			}
		}),
		new OptimizeDependencyReferencedExportsPlugin([
			['ui-lib',{
				treeshake:true,
				// usedExports: ['Badge']
			}]
		])
	]
};
