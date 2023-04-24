/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	mode: "development",
	entry: { main: "./src/index.tsx" },
	builtins: {
		html: [{ template: "./index.html" }],
		define: {
			"process.env.NODE_ENV": "'development'"
		}
	}
};
