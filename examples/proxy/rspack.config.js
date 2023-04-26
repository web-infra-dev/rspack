const path = require("path");
module.exports = (env, argv) => {
	console.log("env:", env, argv);
	return {
		context: __dirname,
		entry: {
			main: "./index.js"
		},
		output: {
			path: path.resolve(__dirname, "dist")
		},
		devServer: {
			proxy: [
				{
					context: ["/api", "/auth"],
					target: "http://localhost:3000"
				}
			]
		}
	};
};
