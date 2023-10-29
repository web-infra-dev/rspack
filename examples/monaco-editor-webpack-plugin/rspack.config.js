const path = require('path');
const MonacoWebpackPlugin = require('monaco-editor-webpack-plugin');

module.exports = {
	entry: {
		app: './index.js',
	},
	output: {
		globalObject: 'self',
		filename: '[name].bundle.js',
		path: path.resolve(__dirname, 'dist')
	},
	module: {
		rules: [
			{
				test: /\.ttf$/,
				type: 'asset/resource',
			}
		]
	},
	builtins: {
		html: [{ template: './index.html' }]
	},
	plugins: [
		new MonacoWebpackPlugin({
			languages: ['typescript', 'javascript', 'css']
		})
	]
};
