/** @type {import("webpack").Configuration} */
module.exports = {
	entry: {
		main: "./src/index.js"
	},
	optimization: {
		concatenateModules: true
	}
};
