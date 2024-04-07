module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [
			{
				filename: "output.html",
				template: "input.html",
				inject: "head",
				scriptLoading: "blocking",
				sri: "sha512"
			}
		]
	}
};
