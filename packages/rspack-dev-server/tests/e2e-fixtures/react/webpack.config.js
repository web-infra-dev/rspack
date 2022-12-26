module.exports = {
	mode: "development",
	entry: "./index.jsx",
	devServer: {
		hot: true
	},
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	builtins: {
		html: [
			{
				template: "./index.html",
				publicPath: "/"
			}
		],
		define: {
			"process.env.NODE_ENV": JSON.stringify("development")
		}
	}
};
