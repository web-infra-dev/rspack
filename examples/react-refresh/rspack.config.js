/**
 * @type {import('@rspack/cli').Configuration}
 */
const config = {
	mode: "development",
	entry: { main: "./src/index.tsx" },
	builtins: {
		html: [{ template: "./index.html" }],
		define: {
			"process.env.NODE_ENV": "'development'"
		},
		treeShaking: 'module'
	},

};
module.exports = config;
