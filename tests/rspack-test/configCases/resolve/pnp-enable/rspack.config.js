/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	entry: {
		main: "./index.js"
	},
	resolve: {
		pnp: true
	}
};
