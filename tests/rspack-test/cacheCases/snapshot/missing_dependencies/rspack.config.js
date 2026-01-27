/** @type {import("@rspack/core").Configuration} */
module.exports = {
	context: __dirname,
	resolve: {
		alias: {
			alias_file: ["./file1", "./file2"]
		}
	},
	experiments: {
		cache: {
			type: "persistent"
		}
	}
};
