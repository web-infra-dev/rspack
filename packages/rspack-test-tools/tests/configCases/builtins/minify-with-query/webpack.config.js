/**@type {import('@rspack/cli').Configuration}*/
module.exports = {
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	output: {
		filename: 'bundle0.js?hash=[contenthash]',
		cssFilename: 'bundle0.css?hash=[contenthash]'
	},
	optimization: {
		minimize: true
	}
};
