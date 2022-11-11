module.exports = {
	entry: {
		index: ["./index.js"]
	},
	output: {
		publicPath: "/base",
	},
	builtins: {
		html: [{
			favicon: "favicon.ico"
		}]
	}
};
