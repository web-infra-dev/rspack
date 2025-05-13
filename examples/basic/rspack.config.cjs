module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	output: {
		publicPath: () => { }
	},
	stats: { all: true }
};
