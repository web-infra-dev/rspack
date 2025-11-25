const { container } = require("@rspack/core");

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
					requiredVersion:'*',
					treeshake:true,
					usedExports: ['Badge']
				},
				'ui-lib2': {
					requiredVersion:'*',
					treeshake:true,
				},
				'ui-lib-side-effect': {
					requiredVersion:'*',
					treeshake:true,
				}
			}
		})
	]
};
