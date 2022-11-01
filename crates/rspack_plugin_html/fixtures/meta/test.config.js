module.exports = {
	entry: {
		index: ["./index.js"]
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
