const path = require('path')

module.exports = {
	findBundle() {
		return ['index.mjs']
	},
	esmLibPluginOptions: {
		preserveModules: path.resolve(__dirname, 'src'),
	},
}
