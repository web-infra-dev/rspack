/** @type {import("@rspack/core").Configuration} */
module.exports = {
	entry() {
		return Promise.resolve({
			bundle0: {
				import: "./index.js",
				layer: "client"
			}
		});
	}
};
