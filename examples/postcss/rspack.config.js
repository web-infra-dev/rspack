/** @type {import('@rspack/cli').Configuration} */
const config = {
	mode: "development",
	entry: "./index.js",
	builtins: {
		html: [{}],
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
module.exports = config;
