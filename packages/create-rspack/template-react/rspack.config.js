const env = process.env.NODE_ENV || "development";
const isDev = env === "development";
/**
 * @type {import('@rspack/cli').Configuration}
 */
module.exports = {
	context: __dirname,
	entry: {
		main: "./src/main.jsx"
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		define: {
			"process.env.NODE_ENV": JSON.stringify(env)
		},
		react: {
			development: isDev,
			refresh: isDev
		}
	}
};
