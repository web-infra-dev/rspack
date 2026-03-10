const path = require('node:path');
const { rspack, experiments } = require('@rspack/core');

const { createPlugins, Layers } = experiments.rsc;
const { ServerPlugin, ClientPlugin } = createPlugins();

const ssrEntry = path.join(__dirname, 'src/framework/entry.ssr.js');
const rscEntry = path.join(__dirname, 'src/framework/entry.rsc.js');

const swcLoaderRule = {
	test: /\.jsx?$/,
	use: [
		{
			loader: 'builtin:swc-loader',
			options: {
				jsc: {
					parser: {
						syntax: 'ecmascript',
						jsx: true,
					},
					transform: {
						react: {
							runtime: 'automatic',
						},
					},
				},
				rspackExperiments: {
					reactServerComponents: true,
				},
			},
		},
	],
};

const cssRule = {
	test: /\.css$/,
	type: 'css/auto',
};

module.exports = [
	{
		mode: 'production',
		target: 'node',
		entry: {
			main: {
				import: ssrEntry,
			},
		},
		resolve: {
			extensions: ['...', '.ts', '.tsx', '.jsx'],
		},
		module: {
			rules: [
				cssRule,
				swcLoaderRule,
				{
					resource: ssrEntry,
					layer: Layers.ssr,
				},
				{
					resource: rscEntry,
					layer: Layers.rsc,
					resolve: {
						conditionNames: ['react-server', '...'],
					},
				},
				{
					issuerLayer: Layers.rsc,
					resolve: {
						conditionNames: ['react-server', '...'],
					},
				},
			],
		},
		plugins: [
			new ServerPlugin({
				onManifest(manifest) {
					expect(manifest).toBeDefined();
					expect(typeof manifest).toBe('object');
					const entryNames = Object.keys(manifest);
					expect(entryNames.length).toBeGreaterThan(0);
					expect(entryNames).toContain('main');

					const mainEntry = manifest.main;
					expect(mainEntry).toHaveProperty('moduleLoading');
					expect(mainEntry.moduleLoading).toHaveProperty('prefix');
					expect(typeof mainEntry.moduleLoading.prefix).toBe('string');
					expect(mainEntry).toHaveProperty('serverManifest');
					expect(mainEntry).toHaveProperty('clientManifest');
					expect(mainEntry).toHaveProperty('serverConsumerModuleMap');
					expect(mainEntry).toHaveProperty('entryCssFiles');
					expect(mainEntry).toHaveProperty('entryJsFiles');

					expect(Object.keys(mainEntry.serverManifest).length).toBe(4);

					const clientPath = path.join(__dirname, './src/Client.js');
					const clientModuleId = './src/Client.js';
					const clientManifestExport = mainEntry.clientManifest[clientPath];
					expect(clientManifestExport.id).toBe(clientModuleId);
					expect(clientManifestExport.name).toBe('*');

					expect(mainEntry.serverConsumerModuleMap[clientModuleId]).toEqual({
						'*': {
							id: '(server-side-rendering)/./src/Client.js',
							name: '*',
							chunks: [],
							async: false,
						},
					});

					// App.js is server entry, so it should be in the entryCssFiles
					const appPath = path.join(__dirname, './src/App.js');
					expect(mainEntry.entryCssFiles[appPath].length).toBe(1);

					expect(mainEntry.entryJsFiles.length).toBe(1);
				},
			}),
			new rspack.DefinePlugin({
				CLIENT_PATH: JSON.stringify(
					path.resolve(__dirname, 'src/Client.js')
				),
			}),
		],
		optimization: {
			moduleIds: 'named',
			chunkIds: 'named',
		},
	},
	{
		mode: 'production',
		target: 'web',
		entry: {
			main: {
				import: './src/framework/entry.client.js',
			},
		},
		resolve: {
			extensions: ['...', '.ts', '.tsx', '.jsx'],
		},
		module: {
			rules: [cssRule, swcLoaderRule],
		},
		plugins: [new ClientPlugin()],
		optimization: {
			moduleIds: 'named',
			chunkIds: 'named',
		},
	},
];
