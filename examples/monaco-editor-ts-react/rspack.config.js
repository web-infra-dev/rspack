const path = require('path');

module.exports = {
	entry: {
		app: './src/index.tsx',
		'editor.worker': 'monaco-editor/esm/vs/editor/editor.worker.js',
		'json.worker': 'monaco-editor/esm/vs/language/json/json.worker',
		'css.worker': 'monaco-editor/esm/vs/language/css/css.worker',
		'html.worker': 'monaco-editor/esm/vs/language/html/html.worker',
		'ts.worker': 'monaco-editor/esm/vs/language/typescript/ts.worker'
	},
	devtool: 'source-map',
	devServer: {
		hot: true,
	},
	resolve: {
		extensions: ['*', '.js', '.jsx', '.tsx', '.ts'],
		tsConfigPath: path.resolve(__dirname, "tsconfig.json")
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
				use: ['file-loader']
			}
		]
	},
	builtins: {
		html: [{ template: './src/index.html' }]
	}
};
