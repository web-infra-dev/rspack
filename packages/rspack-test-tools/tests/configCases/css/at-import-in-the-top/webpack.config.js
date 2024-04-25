module.exports = {
	entry: {
		main: "./index.js"
	},
	module: {
		generator: {
			"css/auto": {
				exportsOnly: false,
			}
		}
	},
	experiments: {
		css: true
	}
};
