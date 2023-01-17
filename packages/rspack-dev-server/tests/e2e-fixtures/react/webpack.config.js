module.exports = {
	mode: "development",
	entry: "./index.jsx",
	devServer: {
		hot: true
	},
	caches: false,
	stats: "none",
	infrastructureLogging: {
		debug: false
	},
	builtins: {
		html: [
			{
				template: "./index.html"
			}
		],
		define: {
			"process.env.NODE_ENV": JSON.stringify("development")
		}
	},
	watchOptions: {
		poll: true
	}
};
