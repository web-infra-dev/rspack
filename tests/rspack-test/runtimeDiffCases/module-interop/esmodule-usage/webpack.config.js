/** @type {import("webpack").Configuration} */
module.exports = {
	mode: "production",
	optimization: {
		usedExports: true,
		minimize: false
	}
};
