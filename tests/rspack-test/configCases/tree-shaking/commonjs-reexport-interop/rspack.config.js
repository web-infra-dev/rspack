/**@type {import("@rspack/core").Configuration}*/
module.exports = {
	mode: "production",
	context: __dirname,
	optimization: {
		moduleIds: "named",
		minimize: false
	}
};
