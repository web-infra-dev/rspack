/** @type {import("@rspack/core").Configuration} */
module.exports = {
	experiments: {
		rspackFuture: {
			bundlerInfo: {
				force: ["version"]
			}
		}
	}
};
