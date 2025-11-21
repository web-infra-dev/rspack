/** @type {import("@rspack/core").Configuration} */
module.exports = {
	target: ["node"],
	entry: {
		main: {
			import: ["./index.js"]
		}
	}
};
