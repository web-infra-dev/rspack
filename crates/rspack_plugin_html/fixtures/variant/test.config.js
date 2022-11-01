module.exports = {
	entry: {
		index: ["./index.js"]
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
