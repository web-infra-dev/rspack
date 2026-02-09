/** @type {import("@rspack/core").Configuration} */
module.exports = {
	mode: "development",
	module: {
		parser: {
			javascript: {
				exportsPresence: 'auto',
			}
		}
	},
};
