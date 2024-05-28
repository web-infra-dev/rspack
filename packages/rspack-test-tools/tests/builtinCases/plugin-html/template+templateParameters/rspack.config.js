/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		index: {
			import: ["./index.js"]
		}
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
