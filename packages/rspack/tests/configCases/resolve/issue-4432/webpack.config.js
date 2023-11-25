/** @type {import('@rspack/cli').Configuration} */
const config = {
	entry: {
		main: "./index.js"
	},
	resolve: {
		mainFields: ["custom", "..."]
	}
};
module.exports = config;
