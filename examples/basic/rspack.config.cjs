module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	module: {
		parser: {
			css: {
				url: false
			},
			"css/auto": {
				url: false
			},
			"css/module": {
				url: false
			}
		}
	},
	experiments: {
		css: true
	}
};
