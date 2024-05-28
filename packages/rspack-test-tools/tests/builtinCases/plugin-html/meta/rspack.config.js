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
				meta: {
					viewport: {
						name: "viewport",
						content: "width=device-width, initial-scale=1, shrink-to-fit=no"
					}
				}
			}
		]
	}
};
