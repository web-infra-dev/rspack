module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
	},
	builtins: {
		html: [
			{
				templateContent:
					"<!DOCTYPE html><html><body><div><%= foo %></div></body></html>",
				templateParameters: {
					foo: "bar"
				}
			}
		]
	}
};
