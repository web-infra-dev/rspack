const path = require("path");

const { sharing } = require("@rspack/core");

const { ShareContainerPlugin, OptimizeDependencyReferencedExportsPlugin,SharePlugin } = sharing;

/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		main: "./index.js"
	},
	plugins: [
		new SharePlugin({
			shared: {
				'ui-lib': {}
			}
		}),
		// new ShareContainerPlugin({
		// 	library: {
		// 		name:'ui_lib',
		// 		type:'commonjs2'
		// 	},
		// 	mfName: "host",
		// 	shareName: "ui-lib",
		// 	version: "1.0.0",
		// 	request: path.resolve(__dirname, "node_modules/ui-lib/index.js")
		// }),
		new OptimizeDependencyReferencedExportsPlugin([
			['ui-lib',{
				treeshake:true,
				usedExports: ['Badge']
			}]
		])
	]
};
