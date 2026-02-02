const rspack = require("@rspack/core");

/** @type {import("@rspack/core").Configuration} */
const baseConfig = {
	mode: "production",
	context: __dirname,
	entry: "./index.jsx",
	experiments: {
		"outputModule": true
	},
	externalsType: "module-import",
	externals: {
		'react': 'react',
		'./App1': './App1',
		'./App2': './App2',
	},
	output: {
		module: true,
		library: {
			type: 'modern-module',
		},
	},
	plugins: [
		new rspack.experiments.RslibPlugin()
	],
	optimization: {
		minimize: false,
	},
	module: {
		parser: {
			javascript: {
				jsx: true,
			},
			'javascript/auto': {
				jsx: true,
			},
			'javascript/dynamic': {
				jsx: true,
			},
			'javascript/esm': {
				jsx: true,
			},
		},
		rules: [
			{
				test: /\.(jsx|tsx)$/,
				loader: "builtin:swc-loader",
				options: {
					jsc: {
						parser: {
							syntax: "typescript",
							tsx: true
						},
						transform: {
							react: {
								runtime: "preserve"
							}
						},
						target: "esnext",
					},
				}
			}
		]
	}
}

/** @type {import("@rspack/core").Configuration[]} */
module.exports = [
	{
		...baseConfig,
		output: {
			...baseConfig.output,
			filename: "bundle0.jsx"
		}
	},
	{
		...baseConfig,
		output: {
			...baseConfig.output,
			filename: "bundle1.jsx"
		},
		optimization: {
			...baseConfig.optimization,
			minimize: true,
			minimizer: [
      new rspack.SwcJsMinimizerRspackPlugin({
				test: /\.jsx?$/,
        minimizerOptions: {
					mangle: false,
					compress: {
						defaults: false,
						unused: true,
						dead_code: true,
					},
					format: {
						comments: 'some',
						preserve_annotations: true,
					},
				}
      })]
		},
	},
	{
		experiments: {
			},
		name: "test-output",
		entry: "./test.js",
		output: {
			...baseConfig.output,
			filename: "test.js"
		}
	}
]
