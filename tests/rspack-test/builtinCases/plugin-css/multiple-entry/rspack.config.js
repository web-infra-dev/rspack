/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry: {
		"main-one": {
			import: ["./index-one.js"]
		},
		"main-two": {
			import: ["./index-two.js"]
		}
	}
};
