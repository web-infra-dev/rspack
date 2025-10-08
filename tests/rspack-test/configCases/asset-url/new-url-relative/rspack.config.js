const rspack = require('@rspack/core');

/**@type {import('@rspack/core').Configuration} */
const common = {
	entry: {
		main: './index.js',
		test: './test.js',
	},
	optimization:{
		splitChunks: {
			cacheGroups: {
				test: {
					chunks: 'all',
					minSize: 0,
					test: /test\.js/,
					name: 'test',
				}
			}
		}
	},
	module: {
		parser: {
			javascript: {
				url: 'new-url-relative',
				importMeta: false,
			}
		},
		generator: {
			asset: {
				filename: 'asset/static-[name].js'
			}
		}
	},
	experiments: {
		outputModule: true,
	},
}

module.exports = [
	{
		...common,
		optimization: {
			...common.optimization,
			concatenateModules: true,
		},
		output: {
			filename: `[name]-0.mjs`
		},
		plugins: [
			new rspack.DefinePlugin({
				INDEX: 0,
			})
		]
	},
	{
		...common,
		optimization: {
			...common.optimization,
			concatenateModules: false,
		},
		output: {
			filename: `[name]-1.mjs`
		},
		plugins: [
			new rspack.DefinePlugin({
				INDEX: 1,
			})
		]
	}
]
