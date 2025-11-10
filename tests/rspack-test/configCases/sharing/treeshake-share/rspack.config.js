const path = require("path");

const { container ,sharing} = require("@rspack/core");

const {OptimizeDependencyReferencedExportsPlugin } = sharing;
const { ModuleFederationPlugin } = container;

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
		new ModuleFederationPlugin({
			name:'treeshake_share',
			manifest: true,
			shared: {
				'ui-lib': {
					requiredVersion:'*'
				}
			}
		}),
		new OptimizeDependencyReferencedExportsPlugin([
			['ui-lib',{
				treeshake:true,
				usedExports: ['Badge']
			}]
		])
	]
};
