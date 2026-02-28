const path = require('path')
/**@type {import('rspack').Configuration} */
module.exports = {
	entry: './src/index.js',
	output: {
		library: {
			type: "modern-module",
			preserveModules: path.resolve(__dirname, 'src'),
		}
	}
}
