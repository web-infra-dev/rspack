/** @type {import('@rspack/cli').Configuration} */
module.exports = {
	context: __dirname,
	mode: "development",
	entry: {
		main: "./src/index.jsx"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	},
	externals: ["react", "react-dom"],
	externalsType: "amd",
	output: {
		library: {
			type: "amd",
			name: "@[name]"
		}
	}
};
