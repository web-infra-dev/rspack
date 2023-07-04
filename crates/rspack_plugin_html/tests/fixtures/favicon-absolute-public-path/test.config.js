module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [{
			publicPath: "/assets/",
			favicon: path.resolve(__dirname, 'favicon.ico')
		}]
	}
};