module.exports ={
	entry: {
		main: {import: './index.js', filename: 'bundle.mjs'},
		json: {import: './index.json', filename: 'json.mjs'},
	},
	output: {
		module: true,
		library: {
			type: 'modern-module',
		},
	},
	module: {
		parser: {
			javascript: {
				importMeta: false
			}
		}
	},
	optimization: {
		concatenateModules: true,
	},
	experiments: {
		outputModule: true
	}
}
