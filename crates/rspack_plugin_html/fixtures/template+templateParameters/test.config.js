module.exports = {
	entry: {
		index: ["./index.js"]
	},
	builtins: {
		html: [
			{
				template: "index.html",
				templateParameters: {
					foo: "bar"
				}
			}
		]
	}
};
