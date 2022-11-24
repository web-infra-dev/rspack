module.exports = {
	entry: {
		main: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [
			{
				template: "index.html"
			}
		]
	}
};
