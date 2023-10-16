module.exports = {
	module: {
		rules: [
			{
				test: /\.js$/,
				loader: "builtin:swc-loader",
				options: {
					// TODO: add back this until `@rspack/plugin-react-refresh` is finished
					// rspackExperiments: {
					// 	react: {
					// 		refresh: false
					// 	}
					// }
				}
			}
		]
	}
};
