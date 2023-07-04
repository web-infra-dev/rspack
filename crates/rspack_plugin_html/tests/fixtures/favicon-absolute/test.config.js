module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [{
			publicPath: "/",
			favicon: path.resolve(__dirname, 'favicon.ico')
		}]
	}
};