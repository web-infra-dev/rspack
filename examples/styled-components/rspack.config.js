/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	entry: {
		main: "./src/index.tsx"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		]
	}
};
