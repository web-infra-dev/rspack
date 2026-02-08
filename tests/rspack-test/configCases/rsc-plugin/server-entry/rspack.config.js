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
            new ServerPlugin(),
            new rspack.DefinePlugin({
                CLIENT_PATH: JSON.stringify(path.resolve(__dirname, 'src/Client.js')),
            }),
        ],
        optimization: {
            moduleIds: 'named',
            concatenateModules: true,
        },
        // TODO: enable lazy compilation when it works with RSC
        lazyCompilation: false,
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
            rules: [
                cssRule,
                swcLoaderRule
            ],
        },
        plugins: [new ClientPlugin()],
        optimization: {
            moduleIds: 'named',
            concatenateModules: true,
        },
        // TODO: enable lazy compilation when it works with RSC
        lazyCompilation: false,
    },
];
