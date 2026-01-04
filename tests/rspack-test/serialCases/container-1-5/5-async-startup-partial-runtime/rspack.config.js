const path = require('path');
const distRoot = path.resolve(
	__dirname,
	'../../../js/serial/container-1-5/5-async-startup-partial-runtime'
);
const remoteOut = path.join(distRoot, '0-container-full');
const remoteContext = path.resolve(__dirname, '../0-container-full');

// Reuse the real remote container config so the case exercises emitted remotes.
const remoteConfigs = require('../0-container-full/rspack.config.js').map(
	config => {
		const isModule = config.experiments && config.experiments.outputModule;
		return {
			...config,
			context: remoteContext,
			output: {
				...config.output,
				path: remoteOut,
				filename: isModule ? 'module/[name].mjs' : '[name].js',
				chunkFilename: isModule ? 'module/[id].mjs' : '[id].js'
			}
		};
	}
);
// eslint-disable-next-line node/no-unpublished-require
const { ModuleFederationPlugin } = require('@rspack/core').container;

const common = {
	entry: {
		host: { import: './host.js', runtime: 'hostRuntime' },
		plain: { import: './plain.js', runtime: 'plainRuntime' }
	},
	optimization: {
		runtimeChunk: 'single'
	}
};

/** @type {ConstructorParameters<typeof ModuleFederationPlugin>[0]} */
const commonMF = {
	runtime: false,
	shared: ['mocked-react']
};

/** @type {import('@rspack/core').Configuration[]} */
module.exports = [
	...remoteConfigs,
	{
		...common,
		target: 'async-node',
		output: {
			filename: '[name].js',
			uniqueName: '5-async-startup-partial-runtime',
			chunkLoading: 'async-node'
		},
		plugins: [
			new ModuleFederationPlugin({
				name: 'container',
				library: { type: 'commonjs-module' },
				filename: 'container.js',
				remotes: {
					containerA: './0-container-full/container.js'
				},
				...commonMF,
				experiments: {
					asyncStartup: true
				}
			})
		]
	}
];
